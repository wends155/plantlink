//! Rhai Scripting Nodes
//!
//! This module provides the `RhaiNode`, which enables user-defined logic
//! using the Rhai scripting language. Each incoming `MessagePayload` is
//! converted to a Rhai `Dynamic` object (Map), processed by a user script,
//! and the result is converted back to a `MessagePayload`.

use super::{NodeBehavior, NodeContext};
use anyhow::Result;
use plantlink_core::MessagePayload;
use rhai::{AST, Dynamic, Engine, Scope};
use std::sync::Arc;

/// A node that executes a Rhai script.
///
/// Scripts must define a `process(msg)` function that takes the incoming
/// message and returns the processed message.
///
/// # Configuration (`data`)
/// - `code`: The Rhai script body. If not provided, defaults to `return msg;`.
///
/// # Example Script
/// ```rhai
/// fn process(msg) {
///     msg.payload = msg.payload * 1.8 + 32.0; // Celsius to Fahrenheit
///     return msg;
/// }
/// ```
///
/// # Payload Serialization
///
/// [`DataValue::Bytes`] payloads are serialized using a unique placeholder string
/// containing the message UUID to prevent heap exhaustion. The placeholder is
/// automatically reverted to the original bitstream after script execution unless
/// explicitly modified by the user script.
pub struct RhaiNode {
    engine: Arc<Engine>,
    ast: Arc<AST>,
}

impl RhaiNode {
    pub fn new(config: &crate::NodeConfig) -> anyhow::Result<Self> {
        let user_script = config
            .data
            .get("code")
            .and_then(|v| v.as_str())
            .unwrap_or("return msg;");

        tracing::info!("RhaiNode: Compiling script: {}", user_script);

        let wrapped_script = format!("fn process(msg) {{\n{user_script}\n}}");

        let mut engine = Engine::new();
        engine.set_max_operations(5000);
        engine.set_max_array_size(100);
        engine.set_max_map_size(100);

        let ast = engine
            .compile(&wrapped_script)
            .map_err(|e| anyhow::anyhow!("Rhai module compilation failed: {e}"))?;

        Ok(Self {
            engine: Arc::new(engine),
            ast: Arc::new(ast),
        })
    }
}

#[async_trait::async_trait]
impl NodeBehavior for RhaiNode {
    async fn start(&mut self, ctx: NodeContext) -> Result<()> {
        ctx.emit_running("Script compiled successfully");
        Ok(())
    }

    async fn receive(
        &mut self,
        _port: usize,
        msg: std::sync::Arc<MessagePayload>,
        ctx: &NodeContext,
    ) -> Result<()> {
        // Convert MessagePayload to Rhai Dynamic (Map)
        // Safety: DataValue::Bytes serializes to O(N) integer arrays in Rhai.
        // We intercept and convert to a descriptive string to protect the heap.
        let mut msg_to_serialize = (*msg).clone();
        let mut original_bytes = None;
        let mut placeholder_str = String::new();

        if let plantlink_core::DataValue::Bytes(b) = &msg_to_serialize.payload {
            placeholder_str = format!("<binary data: {} bytes [{}]>", b.len(), msg_to_serialize.id);
            original_bytes = Some(b.clone());
            msg_to_serialize.payload = plantlink_core::DataValue::String(placeholder_str.clone());
        }

        let dynamic_msg = match rhai::serde::to_dynamic(&msg_to_serialize) {
            Ok(d) => d,
            Err(e) => {
                ctx.emit_log(format!("RhaiNode [{}]: Serialization Error: {}", ctx.id, e));
                return Ok(());
            }
        };

        // Offload synchronous Rhai execution to a blocking task to avoid stalling the async runner.
        let engine = self.engine.clone();
        let ast = self.ast.clone();

        let result_dynamic = tokio::task::spawn_blocking(move || {
            let mut scope = Scope::new();
            engine.call_fn::<Dynamic>(&mut scope, &ast, "process", (dynamic_msg,))
        })
        .await
        .map_err(|e| anyhow::anyhow!("Task join error: {e}"))?;

        match result_dynamic {
            Ok(result_dynamic) => {
                // Check if result is what we expect (MessagePayload or Map)
                match rhai::serde::from_dynamic::<MessagePayload>(&result_dynamic) {
                    Ok(mut result_msg) => {
                        // Restore original bytes if the script hasn't modified the placeholder string.
                        if let Some(bytes) = original_bytes
                            && let plantlink_core::DataValue::String(s) = &result_msg.payload
                            && s == &placeholder_str
                        {
                            result_msg.payload = plantlink_core::DataValue::Bytes(bytes);
                        }

                        if let Err(e) = ctx.send_output(result_msg).await {
                            tracing::warn!(node_id = %ctx.id, "Failed to send output: {}", e);
                        }
                    }
                    Err(e) => {
                        ctx.emit_log(format!(
                            "RhaiNode [{}]: Return type mismatch. Script must return MessagePayload msg. Error: {}",
                            ctx.id, e
                        ));
                        ctx.emit_error(&format!("Return Type Mismatch: {e}"));
                        return Err(anyhow::anyhow!("Type error: {e}"));
                    }
                }
            }
            Err(e) => {
                ctx.emit_log(format!("RhaiNode [{}]: Runtime Error: {}", ctx.id, e));
                ctx.emit_error(&format!("Runtime Error: {e}"));
                return Err(anyhow::anyhow!("Rhai error: {e}"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::RhaiNode;
    use crate::NodeConfig;
    use crate::nodes::{NodeBehavior, NodeContext, OutputMap};
    use plantlink_core::MessagePayload;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::{RwLock, broadcast, mpsc};
    use tokio_util::sync::CancellationToken;

    type CtxOutputStreams = (
        NodeContext,
        mpsc::Receiver<(usize, std::sync::Arc<MessagePayload>)>,
        tokio::sync::broadcast::Receiver<super::super::SystemEvent>,
    );

    fn make_node(script: Option<&str>) -> RhaiNode {
        let data = if let Some(s) = script {
            serde_json::json!({ "code": s })
        } else {
            serde_json::json!({})
        };
        RhaiNode::new(&NodeConfig {
            id: "r1".into(),
            type_: "rhai".into(),
            data,
        })
        .expect("Failed to create RhaiNode in test")
    }

    fn make_ctx_with_output(id: &str) -> CtxOutputStreams {
        let (tx, rx) = mpsc::channel(16);
        let (sys_tx, sys_rx) = broadcast::channel(32);
        let mut outputs: OutputMap = HashMap::new();
        outputs.insert(0, vec![(tx, 0)]);
        let ctx = NodeContext::new(
            id.to_string(),
            outputs,
            Arc::new(RwLock::new(HashMap::new())),
            sys_tx,
            CancellationToken::new(),
            tokio_util::task::TaskTracker::new(),
        );
        (ctx, rx, sys_rx)
    }

    #[tokio::test]
    async fn test_rhai_passthrough_script() {
        let mut node = make_node(Some("return msg;"));
        let (ctx, mut rx, _sys_rx) = make_ctx_with_output("r1");
        let msg = MessagePayload::default();
        let expected_id = msg.id;
        node.receive(0, std::sync::Arc::new(msg), &ctx)
            .await
            .unwrap();
        let (port, received) = rx.recv().await.expect("Expected output");
        assert_eq!(port, 0);
        assert_eq!(received.id, expected_id);
    }

    #[test]
    fn test_rhai_compile_error_on_new() {
        let config = crate::NodeConfig {
            id: "r1".into(),
            type_: "rhai".into(),
            data: serde_json::json!({ "code": "{{{{ invalid syntax" }),
        };
        let result = RhaiNode::new(&config);
        assert!(result.is_err(), "Expected Err at instantiation");
        if let Err(e) = result {
            assert!(
                e.to_string().contains("Rhai module compilation failed"),
                "Expected compile error message"
            );
        }
    }

    #[tokio::test]
    async fn test_rhai_runtime_error_on_input() {
        let mut node = make_node(Some("throw \"boom\";"));
        let (ctx, mut _rx, mut sys_rx) = make_ctx_with_output("r1");
        let result = node
            .receive(0, std::sync::Arc::new(MessagePayload::default()), &ctx)
            .await;
        assert!(result.is_err(), "Expected Err on runtime error");
        // Drain broadcasts and check for runtime error log
        let mut found_error = false;
        while let Ok(msg) = sys_rx.try_recv() {
            if let crate::nodes::SystemEvent::Log { message } = msg
                && message.contains("Runtime Error")
            {
                found_error = true;
            }
        }
        assert!(found_error, "Expected 'Runtime Error' in broadcast log");
    }

    #[tokio::test]
    async fn test_rhai_default_script_is_passthrough() {
        // No "code" key → defaults to "return msg;"
        let mut node = make_node(None);
        let (ctx, mut rx, _sys_rx) = make_ctx_with_output("r1");
        let msg = MessagePayload::default();
        let expected_id = msg.id;
        node.receive(0, std::sync::Arc::new(msg), &ctx)
            .await
            .unwrap();
        let (_, received) = rx
            .recv()
            .await
            .expect("Expected output from default passthrough");
        assert_eq!(received.id, expected_id);
    }

    #[tokio::test]
    async fn test_rhai_infinite_loop_terminates() {
        // Create an infinite loop script
        let mut node = make_node(Some("let x = 0; loop { x += 1; }"));
        let (ctx, _rx, _sys_rx) = make_ctx_with_output("r1");

        let msg = MessagePayload::default();
        let res = node.receive(0, std::sync::Arc::new(msg), &ctx).await;

        assert!(res.is_err(), "Infinite loop should trigger error, not hang");
        let err_msg = res.unwrap_err().to_string();
        assert!(
            err_msg.contains("Too many operations"),
            "Error should mention operation limit. Got: {err_msg}"
        );
    }

    #[tokio::test]
    async fn test_rhai_binary_passthrough() {
        let mut node = make_node(Some("return msg;"));
        let (ctx, mut rx, _sys_rx) = make_ctx_with_output("r1");

        let mut msg = MessagePayload::default();
        let raw_data = vec![1, 2, 3, 4, 5];
        msg.payload = plantlink_core::DataValue::Bytes(raw_data.clone().into());

        node.receive(0, std::sync::Arc::new(msg), &ctx)
            .await
            .unwrap();

        let (_, received) = rx.recv().await.expect("Expected output");

        if let plantlink_core::DataValue::Bytes(received_bytes) = &received.payload {
            assert_eq!(received_bytes.as_ref(), &raw_data);
        } else {
            panic!("Expected Bytes payload, got: {:?}", received.payload);
        }
    }

    #[tokio::test]
    async fn test_rhai_placeholder_string_not_corrupted() {
        // This script explicitly returns the OLD deterministic placeholder string.
        // In the current buggy version, the engine will see this string, match it,
        // and incorrectly substitute the original bytes from the input message.
        let mut node = make_node(Some(
            "msg.payload = \"<binary data: 5 bytes>\"; return msg;",
        ));
        let (ctx, mut rx, _sys_rx) = make_ctx_with_output("r1");

        let mut msg = MessagePayload::default();
        let raw_data = vec![1, 2, 3, 4, 5];
        msg.payload = plantlink_core::DataValue::Bytes(raw_data.into());

        node.receive(0, std::sync::Arc::new(msg), &ctx)
            .await
            .unwrap();

        let (_, received) = rx.recv().await.expect("Expected output");

        // ASSERT: Payload should be the STRING we explicitly set in the script,
        // not corrupted back into the original BYTES.
        if let plantlink_core::DataValue::String(s) = &received.payload {
            assert_eq!(s, "<binary data: 5 bytes>");
        } else {
            panic!(
                "Expected String payload (explicit script return), got: {:?}",
                received.payload
            );
        }
    }
}
