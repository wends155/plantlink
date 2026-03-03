use crate::NodeConfig;
use crate::nodes::NodeBehavior;
use anyhow::{Result, bail};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::RwLock;

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

    pub fn register<F>(&mut self, type_name: &str, factory: F) -> Result<()>
    where
        F: Fn(&NodeConfig) -> Box<dyn NodeBehavior> + Send + Sync + 'static,
    {
        self.factories.insert(type_name.to_string(), Box::new(factory));
        tracing::info!("Registered node type (instance): {}", type_name);
        Ok(())
    }

    pub fn create(&self, type_name: &str, config: &NodeConfig) -> Result<Box<dyn NodeBehavior>> {
        match self.factories.get(type_name) {
            Some(factory) => Ok(factory(config)),
            None => bail!("Unknown node type: {type_name}"),
        }
    }
}#[cfg(test)]
mod tests {
    use super::*;

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
}
