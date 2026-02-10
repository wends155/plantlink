use crate::NodeConfig;
use crate::nodes::NodeBehavior;
use anyhow::{Result, bail};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

/// Type definition for a node factory function
pub type NodeFactory = Box<dyn Fn(&NodeConfig) -> Box<dyn NodeBehavior> + Send + Sync>;

/// Global registry mapping Node Type String -> Factory Function
static NODE_REGISTRY: Lazy<RwLock<HashMap<String, NodeFactory>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub fn register_node<F>(type_name: &str, factory: F)
where
    F: Fn(&NodeConfig) -> Box<dyn NodeBehavior> + Send + Sync + 'static,
{
    let mut registry = NODE_REGISTRY.write().expect("Registry lock poisoned");
    registry.insert(type_name.to_string(), Box::new(factory));
    tracing::info!("Registered node type: {}", type_name);
}

pub fn create_node(type_name: &str, config: &NodeConfig) -> Result<Box<dyn NodeBehavior>> {
    let registry = NODE_REGISTRY.read().expect("Registry lock poisoned");
    match registry.get(type_name) {
        Some(factory) => Ok(factory(config)),
        None => bail!("Unknown node type: {}", type_name),
    }
}
