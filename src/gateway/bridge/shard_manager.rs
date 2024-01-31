use std::num::NonZeroU16;
use std::sync::Arc;
#[cfg(feature = "framework")]
use std::sync::OnceLock;
use std::time::Duration;

use futures::channel::mpsc::{self, Receiver, Sender, UnboundedSender};
use futures::SinkExt as _;
use tokio::time::timeout;
use tracing::{info, warn};

#[cfg(feature = "voice")]
use super::VoiceGatewayManager;
use super::{ShardId, ShardQueue, ShardQueuer, ShardQueuerMessage, ShardRunnerInfo};
#[cfg(feature = "cache")]
use crate::cache::Cache;
use crate::client::{EventHandler, RawEventHandler};
#[cfg(feature = "framework")]
use crate::framework::Framework;
use crate::gateway::{ConnectionStage, GatewayError, PresenceData};
use crate::http::Http;
use crate::internal::prelude::*;
use crate::internal::tokio::spawn_named;
use crate::model::gateway::GatewayIntents;

/// A manager for handling the status of shards by starting them, restarting them, and stopping
/// them when required.
///
/// **Note**: The [`Client`] internally uses a shard manager. If you are using a Client, then you
/// do not need to make one of these.
///
/// # Examples
///
/// Initialize a shard manager for shards 0 through 2, of 5 total shards:
///
/// ```rust,no_run
/// # use std::error::Error;
/// #
/// # #[cfg(feature = "voice")]
/// # use serenity::model::id::UserId;
/// # #[cfg(feature = "cache")]
/// # use serenity::cache::Cache;
/// #
/// # #[cfg(feature = "framework")]
/// # async fn run() -> Result<(), Box<dyn Error>> {
/// #
/// use std::env;
/// use std::sync::{Arc, OnceLock};
///
/// use serenity::client::{EventHandler, RawEventHandler};
/// use serenity::gateway::{ShardManager, ShardManagerOptions};
/// use serenity::http::Http;
/// use serenity::model::gateway::GatewayIntents;
/// use serenity::prelude::*;
/// use tokio::sync::{Mutex, RwLock};
///
/// struct Handler;
///
/// impl EventHandler for Handler {}
/// impl RawEventHandler for Handler {}
///
/// # let http: Arc<Http> = unimplemented!();
/// let gateway_info = http.get_bot_gateway().await?;
///
/// let data = Arc::new(());
/// let shard_total = gateway_info.shards;
/// let ws_url = Arc::from(gateway_info.url);
/// let event_handler = Arc::new(Handler) as Arc<dyn EventHandler>;
/// let max_concurrency = std::num::NonZeroU16::MIN;
///
/// ShardManager::new(ShardManagerOptions {
///     data,
///     event_handlers: vec![event_handler],
///     raw_event_handlers: vec![],
///     framework: Arc::new(OnceLock::new()),
///     # #[cfg(feature = "voice")]
///     # voice_manager: None,
///     ws_url,
///     shard_total,
///     # #[cfg(feature = "cache")]
///     # cache: unimplemented!(),
///     # http,
///     intents: GatewayIntents::non_privileged(),
///     presence: None,
///     max_concurrency,
/// });
/// # Ok(())
/// # }
/// ```
///
/// [`Client`]: crate::Client
#[derive(Debug)]
pub struct ShardManager {
    return_value_tx: parking_lot::Mutex<Option<Sender<Result<(), GatewayError>>>>,
    /// The shard runners currently managed.
    ///
    /// **Note**: It is highly unrecommended to mutate this yourself unless you need to. Instead
    /// prefer to use methods on this struct that are provided where possible.
    pub runners: Arc<dashmap::DashMap<ShardId, ShardRunnerInfo>>,
    shard_queuer: UnboundedSender<ShardQueuerMessage>,
    gateway_intents: GatewayIntents,
}

impl ShardManager {
    /// Creates a new shard manager, returning both the manager and a monitor for usage in a
    /// separate thread.
    #[must_use]
    pub fn new(opt: ShardManagerOptions) -> (Arc<Self>, Receiver<Result<(), GatewayError>>) {
        let (return_value_tx, return_value_rx) = mpsc::channel(1);
        let (shard_queue_tx, shard_queue_rx) = mpsc::unbounded();

        let runners = Arc::new(dashmap::DashMap::new());

        let manager = Arc::new(Self {
            return_value_tx: parking_lot::Mutex::new(Some(return_value_tx)),
            shard_queuer: shard_queue_tx,
            runners: Arc::clone(&runners),
            gateway_intents: opt.intents,
        });

        let mut shard_queuer = ShardQueuer {
            data: opt.data,
            event_handlers: opt.event_handlers,
            raw_event_handlers: opt.raw_event_handlers,
            #[cfg(feature = "framework")]
            framework: opt.framework,
            last_start: None,
            manager: Arc::clone(&manager),
            queue: ShardQueue::new(opt.max_concurrency),
            runners,
            rx: shard_queue_rx,
            #[cfg(feature = "voice")]
            voice_manager: opt.voice_manager,
            ws_url: opt.ws_url,
            shard_total: opt.shard_total,
            #[cfg(feature = "cache")]
            cache: opt.cache,
            http: opt.http,
            intents: opt.intents,
            presence: opt.presence,
        };

        spawn_named("shard_queuer::run", async move {
            shard_queuer.run().await;
        });

        (Arc::clone(&manager), return_value_rx)
    }

    /// Returns whether the shard manager contains either an active instance of a shard runner
    /// responsible for the given ID.
    ///
    /// If a shard has been queued but has not yet been initiated, then this will return `false`.
    pub fn has(&self, shard_id: ShardId) -> bool {
        self.runners.contains_key(&shard_id)
    }

    /// Initializes all shards that the manager is responsible for.
    ///
    /// This will communicate shard boots with the [`ShardQueuer`] so that they are properly
    /// queued.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub fn initialize(&self, shard_index: u16, shard_init: u16, shard_total: NonZeroU16) {
        let shard_to = shard_index + shard_init;

        self.set_shard_total(shard_total);
        for shard_id in shard_index..shard_to {
            self.boot(ShardId(shard_id), true);
        }
    }

    /// Restarts a shard runner.
    ///
    /// This sends a shutdown signal to a shard's associated [`ShardRunner`], and then queues a
    /// initialization of a shard runner for the same shard via the [`ShardQueuer`].
    ///
    /// # Examples
    ///
    /// Restarting a shard by ID:
    ///
    /// ```rust,no_run
    /// use serenity::model::id::ShardId;
    /// use serenity::prelude::*;
    ///
    /// # async fn run(client: Client) {
    /// // restart shard ID 7
    /// client.shard_manager.restart(ShardId(7)).await;
    /// # }
    /// ```
    ///
    /// [`ShardRunner`]: super::ShardRunner
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn restart(&self, shard_id: ShardId) {
        info!("Restarting shard {shard_id}");
        self.shutdown(shard_id, 4000).await;
        self.boot(shard_id, false);
    }

    /// Returns the [`ShardId`]s of the shards that have been instantiated and currently have a
    /// valid [`ShardRunner`].
    ///
    /// [`ShardRunner`]: super::ShardRunner
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub fn shards_instantiated(&self) -> Vec<ShardId> {
        self.runners.iter().map(|v| *v.key()).collect()
    }

    /// Attempts to shut down the shard runner by Id.
    ///
    /// Returns a boolean indicating whether a shard runner was present. This is _not_ necessary an
    /// indicator of whether the shard runner was successfully shut down.
    ///
    /// **Note**: If the receiving end of an mpsc channel - theoretically owned by the shard runner
    /// - no longer exists, then the shard runner will not know it should shut down. This _should
    /// never happen_. It may already be stopped.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn shutdown(&self, shard_id: ShardId, code: u16) {
        const TIMEOUT: tokio::time::Duration = tokio::time::Duration::from_secs(5);

        info!("Shutting down shard {}", shard_id);

        let (finished_channel_tx, finished_channel_rx) = futures::channel::oneshot::channel();
        let msg = ShardQueuerMessage::ShutdownShard {
            resp: finished_channel_tx,
            shard_id,
            code,
        };

        if self.shard_queuer.unbounded_send(msg).is_ok() {
            if let Err(err) = timeout(TIMEOUT, finished_channel_rx).await {
                warn!("Failed to cleanly shutdown shard {shard_id}, reached timeout: {err:?}");
            }
        }

        self.runners.remove(&shard_id);
    }

    /// Sends a shutdown message for all shards that the manager is responsible for that are still
    /// known to be running.
    ///
    /// If you only need to shutdown a select number of shards, prefer looping over the
    /// [`Self::shutdown`] method.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn shutdown_all(&self) {
        let keys = {
            if self.runners.is_empty() {
                return;
            }

            self.runners.iter().map(|v| *v.key()).collect::<Vec<_>>()
        };

        info!("Shutting down all shards");

        for shard_id in keys {
            self.shutdown(shard_id, 1000).await;
        }

        drop(self.shard_queuer.unbounded_send(ShardQueuerMessage::Shutdown));

        // this message is received by Client::start_connection, which lets the main thread know
        // and finally return from Client::start
        self.return_with_value(Ok(())).await;
    }

    fn set_shard_total(&self, shard_total: NonZeroU16) {
        info!("Setting shard total to {shard_total}");

        let msg = ShardQueuerMessage::SetShardTotal(shard_total);
        drop(self.shard_queuer.unbounded_send(msg));
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    fn boot(&self, shard_id: ShardId, concurrent: bool) {
        info!("Telling shard queuer to start shard {shard_id}");

        drop(self.shard_queuer.unbounded_send(ShardQueuerMessage::Start {
            shard_id,
            concurrent,
        }));
    }

    /// Returns the gateway intents used for this gateway connection.
    #[must_use]
    pub fn intents(&self) -> GatewayIntents {
        self.gateway_intents
    }

    pub async fn return_with_value(&self, ret: Result<(), GatewayError>) {
        let Some(mut return_value_tx) = self.return_value_tx.lock().take() else {
            tracing::warn!("failed to send return value as value has already been sent");
            return;
        };

        if let Err(e) = return_value_tx.send(ret).await {
            tracing::warn!("failed to send return value: {}", e);
        }
    }

    pub fn update_shard_latency_and_stage(
        &self,
        id: ShardId,
        latency: Option<Duration>,
        stage: ConnectionStage,
    ) {
        if let Some(mut runner) = self.runners.get_mut(&id) {
            runner.latency = latency;
            runner.stage = stage;
        }
    }
}

impl Drop for ShardManager {
    /// A custom drop implementation to clean up after the manager.
    ///
    /// This shuts down all active [`ShardRunner`]s and attempts to tell the [`ShardQueuer`] to
    /// shutdown.
    ///
    /// [`ShardRunner`]: super::ShardRunner
    fn drop(&mut self) {
        drop(self.shard_queuer.unbounded_send(ShardQueuerMessage::Shutdown));
    }
}

pub struct ShardManagerOptions {
    pub data: Arc<dyn std::any::Any + Send + Sync>,
    pub event_handlers: Vec<Arc<dyn EventHandler>>,
    pub raw_event_handlers: Vec<Arc<dyn RawEventHandler>>,
    #[cfg(feature = "framework")]
    pub framework: Arc<OnceLock<Arc<dyn Framework>>>,
    #[cfg(feature = "voice")]
    pub voice_manager: Option<Arc<dyn VoiceGatewayManager>>,
    pub ws_url: Arc<str>,
    pub shard_total: NonZeroU16,
    #[cfg(feature = "cache")]
    pub cache: Arc<Cache>,
    pub http: Arc<Http>,
    pub intents: GatewayIntents,
    pub presence: Option<PresenceData>,
    pub max_concurrency: NonZeroU16,
}
