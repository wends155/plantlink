use super::{NodeBehavior, NodeContext};
use plantlink_core::MessagePayload;
use anyhow::Result;
use rhai::{Engine, Scope, AST, Dynamic};

pub struct RhaiNode {
    engine: Engine,
    ast: Option<AST>,
    compile_error: Option<String>,
}

impl RhaiNode {
    pub fn new(config: &crate::NodeConfig) -> Self {
        let user_script = config.data.get("code").and_then(|v| v.as_str()).unwrap_or("return msg;");
        
        tracing::info!("RhaiNode: Compiling script: {}", user_script);

        // Implicitly wrap user code in a function
        let wrapped_script = format!(
            "fn process(msg) {{\n{}\n}}", 
            user_script
        );

        let engine = Engine::new();
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
            let json_log = serde_json::json!({ "type": "log", "message": log_msg }).to_string();
            let _ = ctx.system_tx.send(json_log);
            
            // Emit Error Status
             let status = super::NodeStatus {
                 node_id: ctx.id.clone(),
                 state: "error".to_string(),
                 message: format!("Compilation Error: {}", err),
             };
             let json_status = serde_json::json!({ "type": "status", "data": status }).to_string();
             let _ = ctx.system_tx.send(json_status);
        }
        Ok(())
    }

    async fn on_input(&mut self, _port: usize, msg: MessagePayload, ctx: NodeContext) -> Result<()> {
        if let Some(ast) = &self.ast {
            let mut scope = Scope::new();
            
            // Convert MessagePayload to Rhai Dynamic (Map)
            let dynamic_msg = match rhai::serde::to_dynamic(&msg) {
                Ok(d) => d,
                Err(e) => {
                    let log = format!("RhaiNode [{}]: Serialization Error: {}", ctx.id, e);
                    let _ = ctx.system_tx.send(log);
                    return Ok(());
                }
            };

            // Call the 'process' function
            // We use call_fn which captures runtime errors (exceptions)
            // Note: call_fn arguments are passed as a tuple
            let _options = rhai::CallFnOptions::new().eval_ast(false); // Do not re-evaluate constants if possible check docs?
            // Actually call_fn on Engine:
            // engine.call_fn(&mut scope, &ast, "process", (dynamic_msg,))
            
            match self.engine.call_fn::<Dynamic>(&mut scope, ast, "process", (dynamic_msg,)) {
                Ok(result_dynamic) => {
                     // Check if result is what we expect (MessagePayload or Map)
                     match rhai::serde::from_dynamic::<MessagePayload>(&result_dynamic) {
                         Ok(result_msg) => {
                             ctx.send_output(result_msg).await;
                         }
                         Err(e) => {
                             // User script returned something else?
                             let log_msg = format!("RhaiNode [{}]: Return type mismatch. Script must return MessagePayload msg. Error: {}", ctx.id, e);
                             let json = serde_json::json!({ "type": "log", "message": log_msg }).to_string();
                             let _ = ctx.system_tx.send(json);
                             
                             // Emit Error Status
                             let status = super::NodeStatus {
                                 node_id: ctx.id.clone(),
                                 state: "error".to_string(),
                                 message: format!("Return Type Mismatch: {}", e),
                             };
                             let json_status = serde_json::json!({ "type": "status", "data": status }).to_string();
                             let _ = ctx.system_tx.send(json_status);
                         }
                     }
                }
                Err(e) => {
                    // Runtime Error in Script
                    let log_msg = format!("RhaiNode [{}]: Runtime Error: {}", ctx.id, e);
                    let json = serde_json::json!({ "type": "log", "message": log_msg }).to_string();
                    let _ = ctx.system_tx.send(json);
                    
                    // Emit Error Status
                     let status = super::NodeStatus {
                         node_id: ctx.id.clone(),
                         state: "error".to_string(),
                         message: format!("Runtime Error: {}", e),
                     };
                     let json_status = serde_json::json!({ "type": "status", "data": status }).to_string();
                     let _ = ctx.system_tx.send(json_status);
                }
            }
        } else {
             // Node is in error state due to compilation failure
             // We already logged in start, but we can log again on attempts to use
             if let Some(err) = &self.compile_error {
                  let log_msg = format!("RhaiNode [{}]: Cannot process input. Compilation failed: {}", ctx.id, err);
                  let json = serde_json::json!({ "type": "log", "message": log_msg }).to_string();
                  let _ = ctx.system_tx.send(json);
             }
        }
        Ok(())
    }
}
