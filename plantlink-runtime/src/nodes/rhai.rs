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
pub struct RhaiNode {
    engine: Engine,
    ast: Option<AST>,
    compile_error: Option<String>,
}

impl RhaiNode {
    pub fn new(config: &crate::NodeConfig) -> Self {
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

        let (ast, compile_error) = match engine.compile(&wrapped_script) {
            Ok(ast) => (Some(ast), None),
            Err(e) => (None, Some(e.to_string())),
        };

        Self {
            engine,
            ast,
            compile_error,
        }
    }
}

#[async_trait::async_trait]
impl NodeBehavior for RhaiNode {
    async fn start(&mut self, ctx: NodeContext) -> Result<()> {
        if let Some(err) = &self.compile_error {
            let log_msg = format!("RhaiNode [{}]: Compilation Error: {}", ctx.id, err);
            let event = super::SystemEvent::Log { message: log_msg };
            if let Err(e) = ctx.system_tx.send(event) {
                tracing::warn!(node_id = %ctx.id, "Failed to broadcast log: {}", e);
            }
            ctx.emit_error(&format!("Compilation Error: {err}"));
            Err(anyhow::anyhow!("Compilation Error: {err}"))
        } else {
            ctx.emit_running("Script compiled successfully");
            Ok(())
        }
    }

    async fn receive(
        &mut self,
        _port: usize,
        msg: std::sync::Arc<MessagePayload>,
        ctx: &NodeContext,
    ) -> Result<()> {
        if let Some(ast) = &self.ast {
            let mut scope = Scope::new();

            // Convert MessagePayload to Rhai Dynamic (Map)
            let dynamic_msg = match rhai::serde::to_dynamic(&*msg) {
                Ok(d) => d,
                Err(e) => {
                    let log = format!("RhaiNode [{}]: Serialization Error: {}", ctx.id, e);
                    let event = super::SystemEvent::Log { message: log };
                    if let Err(e) = ctx.system_tx.send(event) {
                        tracing::warn!(node_id = %ctx.id, "Failed to broadcast log: {}", e);
                    }
                    return Ok(());
                }
            };

            // Call the 'process' function
            match self
                .engine
                .call_fn::<Dynamic>(&mut scope, ast, "process", (dynamic_msg,))
            {
                Ok(result_dynamic) => {
                    // Check if result is what we expect (MessagePayload or Map)
                    match rhai::serde::from_dynamic::<MessagePayload>(&result_dynamic) {
                        Ok(result_msg) => {
                            if let Err(e) = ctx.send_output(result_msg).await {
                                tracing::warn!(node_id = %ctx.id, "Failed to send output: {}", e);
                            }
                        }
                        Err(e) => {
                            let log_msg = format!(
                                "RhaiNode [{}]: Return type mismatch. Script must return MessagePayload msg. Error: {}",
                                ctx.id, e
                            );
                            let event = super::SystemEvent::Log { message: log_msg };
                            if let Err(e) = ctx.system_tx.send(event) {
                                tracing::warn!(node_id = %ctx.id, "Failed to broadcast log: {}", e);
                            }
                            ctx.emit_error(&format!("Return Type Mismatch: {e}"));
                            return Err(anyhow::anyhow!("Type error: {e}"));
                        }
                    }
                }
                Err(e) => {
                    let log_msg = format!("RhaiNode [{}]: Runtime Error: {}", ctx.id, e);
                    let event = super::SystemEvent::Log { message: log_msg };
                    if let Err(e) = ctx.system_tx.send(event) {
                        tracing::warn!(node_id = %ctx.id, "Failed to broadcast log: {}", e);
                    }
                    ctx.emit_error(&format!("Runtime Error: {e}"));
                    return Err(anyhow::anyhow!("Rhai error: {e}"));
                }
            }
        } else {
            // Node is in error state due to compilation failure
            if let Some(err) = &self.compile_error {
                let log_msg = format!(
                    "RhaiNode [{}]: Cannot process input. Compilation failed: {}",
                    ctx.id, err
                );
                let event = super::SystemEvent::Log { message: log_msg };
                if let Err(e) = ctx.system_tx.send(event) {
                    tracing::warn!(node_id = %ctx.id, "Failed to broadcast log: {}", e);
                }
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
        broadcast::Receiver<super::super::SystemEvent>,
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
        );
        (ctx, rx, sys_rx)
    }

    #[tokio::test]
    async fn test_rhai_passthrough_script() {
        let mut node = make_node(Some("return msg;"));
        let (ctx, mut rx, _sys_rx) = make_ctx_with_output("r1");
        let msg = MessagePayload::default();
        let expected_id = msg.id.clone();
        node.receive(0, std::sync::Arc::new(msg), &ctx)
            .await
            .unwrap();
        let (port, received) = rx.recv().await.expect("Expected output");
        assert_eq!(port, 0);
        assert_eq!(received.id, expected_id);
    }

    #[tokio::test]
    async fn test_rhai_compile_error_on_start() {
        let mut node = make_node(Some("{{{{ invalid syntax"));
        let (ctx, _sys_rx) = NodeContext::for_test("r1");
        let result = node.start(ctx).await;
        assert!(result.is_err(), "Expected Err on compile error");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Compilation Error"),
            "Expected compile error message, got: {err_msg}"
        );
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
        let expected_id = msg.id.clone();
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
}
