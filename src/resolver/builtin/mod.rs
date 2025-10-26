use std::sync::Arc;

use crate::resolver::registry::register;

mod youtube;
pub use youtube::YouTubeResolver;

/// registers builtin resolvers
pub fn register_builtins() {
    register(Arc::new(YouTubeResolver));
}
