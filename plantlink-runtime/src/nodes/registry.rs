use crate::NodeConfig;
use crate::nodes::NodeBehavior;
use anyhow::{Result, bail};
use std::collections::HashMap;

/// Type definition for a node factory function
pub type NodeFactory = Box<dyn Fn(&NodeConfig) -> Box<dyn NodeBehavior> + Send + Sync>;

/// A registry that dynamically maps string identifiers to node instantiation functions.
///
/// The registry allows the runtime engine to dynamically deserialize node types from JSON
/// configurations and instantiate the concrete structs that implement `NodeBehavior`.
#[derive(Default)]
pub struct NodeRegistry {
    factories: HashMap<String, NodeFactory>,
}

impl NodeRegistry {
    /// Creates a new, empty node registry.
    ///
    /// # Returns
    ///
    /// A clean `NodeRegistry` instance with no recorded factories.
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    /// Registers a new node factory function.
    ///
    /// # Arguments
    ///
    /// * `type_name` - The string identifier used in the flow configuration JSON.
    /// * `factory` - A closure or function that takes a `NodeConfig` and returns a boxed `NodeBehavior`.
    ///
    /// # Returns
    ///
    /// `Ok(())` upon successful insertion.
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

    /// Dynamically instantiates a node using the registered factory.
    ///
    /// # Arguments
    ///
    /// * `type_name` - The identifier of the node type to create.
    /// * `config` - The configuration containing instance-specific data.
    ///
    /// # Returns
    ///
    /// The completely initialized node implementing `NodeBehavior`.
    ///
    /// # Errors
    ///
    /// Returns an error if the `type_name` doesn't exist in the registry.
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
