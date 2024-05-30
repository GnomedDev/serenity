#[cfg(feature = "cache")]
pub use crate::cache::Cache;
use crate::gateway::ShardManager;
use crate::http::Http;
use crate::model::prelude::*;

#[non_exhaustive]
pub struct ClientContext {
    pub shard_manager: ShardManager,
    #[cfg(feature = "cache")]
    pub cache: Cache,
    pub http: Http,
}

/// EventContext is a container for event specific data passed to all [`EventHandler`] methods.
#[derive(Clone)]
#[non_exhaustive]
pub struct EventContext {
    /// The [`ShardId`] that this event was triggered from.
    ///
    /// Actions on the shard can be performed by looking up the [`ShardMessenger`] in
    /// [`ShardManager`], usually reachable via [`ClientContext::shard_manager`].
    pub shard_id: ShardId,
}
