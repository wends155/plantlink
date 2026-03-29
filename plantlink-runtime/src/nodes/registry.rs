use crate::NodeConfig;
use crate::nodes::NodeBehavior;
use anyhow::{Result, bail};
use std::collections::HashMap;

/// Type definition for a node factory function
pub type NodeFactory = Box<dyn Fn(&NodeConfig) -> Box<dyn NodeBehavior> + Send + Sync>;

#[derive(Default)]
pub struct NodeRegistry {
    factories: HashMap<String, NodeFactory>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    #[allow(clippy::unnecessary_wraps)] // callers use `?` in register_defaults; keep Result for API flexibility
    pub fn register<F>(&mut self, type_name: &str, factory: F) -> Result<()>
    where
        F: Fn(&NodeConfig) -> Box<dyn NodeBehavior> + Send + Sync + 'static,
    {
        self.factories
            .insert(type_name.to_string(), Box::new(factory));
        tracing::info!("Registered node type (instance): {}", type_name);
        Ok(())
    }

    pub fn create(&self, type_name: &str, config: &NodeConfig) -> Result<Box<dyn NodeBehavior>> {
        match self.factories.get(type_name) {
            Some(factory) => Ok(factory(config)),
            None => bail!("Unknown node type: {type_name}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NodeRegistry;

    #[test]
    fn test_instance_registry_unknown_type() {
        let registry = NodeRegistry::new();
        let config = crate::NodeConfig {
            id: "n2".into(),
            type_: "does-not-exist".into(),
            data: serde_json::json!({}),
        };
        match registry.create("does-not-exist", &config) {
            Err(e) => assert!(e.to_string().contains("Unknown node type")),
            Ok(_) => panic!("Expected error, got Ok"),
        }
    }

    #[test]
    fn test_registry_create_known_type() {
        let mut registry = NodeRegistry::new();
        let config = crate::NodeConfig {
            id: "n1".into(),
            type_: "test-node".into(),
            data: serde_json::json!({}),
        };
        registry
            .register("test-node", |cfg| {
                Box::new(crate::nodes::console::ConsoleNode::new(cfg))
            })
            .unwrap();
        let result = registry.create("test-node", &config);
        assert!(result.is_ok(), "Expected Ok from known type");
    }

    #[test]
    fn test_registry_register_defaults() {
        let mut registry = NodeRegistry::new();

        crate::nodes::register_defaults(&mut registry).unwrap();
        let dummy_cfg = crate::NodeConfig {
            id: "n".into(),
            type_: "x".into(),
            data: serde_json::json!({}),
        };
        for type_name in &[
            "inject",
            "console",
            "nats-broker",
            "nats-sub",
            "nats-pub",
            "rhai",
            "function",
            "rhai-function",
        ] {
            let result = registry.create(type_name, &dummy_cfg);
            assert!(
                result.is_ok(),
                "Expected '{type_name}' to be registered, got: {:?}",
                result.err()
            );
        }
    }
}
