use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};

use super::traits::MediaResolver;

static REGISTRY: Lazy<RwLock<Vec<Arc<dyn MediaResolver>>>> = Lazy::new(|| RwLock::new(Vec::new()));

/// register a resolver at runtime (library users can call this in their init)
pub fn register(resolver: Arc<dyn MediaResolver>) {
    REGISTRY.write().unwrap().push(resolver);
}

/// iterate current resolvers (builtins + user-registered)
pub fn resolvers() -> Vec<Arc<dyn MediaResolver>> {
    REGISTRY.read().unwrap().clone()
}
