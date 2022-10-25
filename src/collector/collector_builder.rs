use std::num::NonZeroU32;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use tokio::time::Sleep;

use super::collectable::Collectable;
use super::filter::{Filter, FilterTrait};
use super::filter_options::CommonFilterOptions;
use super::{Collector, CollectorCallback, CollectorError, FilterFn, ReactionAction};
use crate::client::bridge::gateway::ShardMessenger;
use crate::model::channel::Message;
use crate::model::event::{Event, EventType, RelatedIdsForEventType};
use crate::model::id::{ChannelId, GuildId, MessageId, UserId};
use crate::model::prelude::{ComponentInteraction, ModalSubmitInteraction};
use crate::{Error, Result};

#[must_use = "Builders must be built"]
pub struct CollectorBuilder<'a, Item: Collectable> {
    common_options: CommonFilterOptions<Item::FilterItem>,
    filter_options: Item::FilterOptions,
    shard_messenger: &'a ShardMessenger,
    timeout: Option<Pin<Box<Sleep>>>,
}

impl<'a, Item: Collectable + 'static> CollectorBuilder<'a, Item> {
    pub fn new(shard_messenger: &'a ShardMessenger) -> Self {
        Self {
            shard_messenger,

            timeout: None,
            common_options: CommonFilterOptions::default(),
            filter_options: Item::FilterOptions::default(),
        }
    }

    pub fn build(self) -> Collector<Item>
    where
        Filter<Item>: FilterTrait<Item> + Send + Sync,
    {
        let (mut filter, recv) = Filter::<Item>::new(self.filter_options, self.common_options);
        self.shard_messenger.add_collector(CollectorCallback(Box::new(move |event| {
            if let Some(item) = Item::extract(event) {
                filter.process_item(item)
            } else {
                false
            }
        })));

        Collector {
            timeout: self.timeout,
            receiver: Box::pin(recv),
        }
    }

    /// Sets a filter function where items passed to the `function` must return `true`,
    /// otherwise the item won't be collected and failed the filter process.
    ///
    /// This is the last instance to pass for an item to count as *collected*.
    pub fn filter(
        mut self,
        function: impl Fn(&Item::FilterItem) -> bool + 'static + Send + Sync,
    ) -> Self {
        self.common_options.filter = Some(FilterFn(Arc::new(function)));

        self
    }

    /// Limits how many items can be collected.
    ///
    /// An item is considered *collected*, if the message passes all the requirements.
    pub fn collect_limit(mut self, limit: u32) -> Self {
        self.common_options.collect_limit = NonZeroU32::new(limit);

        self
    }

    /// Limits how many events will attempt to be filtered.
    pub fn filter_limit(mut self, limit: u32) -> Self {
        self.common_options.filter_limit = NonZeroU32::new(limit);

        self
    }

    /// Sets a [`Duration`] for how long the collector shall receive events.
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(Box::pin(tokio::time::sleep(duration)));

        self
    }
}

impl<Item: Collectable + Send + Sync + 'static> CollectorBuilder<'_, Item>
where
    Filter<Item>: FilterTrait<Item> + Send + Sync,
{
    pub async fn collect_single(self) -> Option<Arc<Item>> {
        let mut collector = self.build();
        collector.next().await
    }
}

macro_rules! gen_macro {
    ($name:ident, $function_name:ident, $type_name:ty) => {
        macro_rules! $name {
            ($doc:literal) => {
                #[doc=$doc]
                pub fn $function_name(mut self, $function_name: $type_name) -> Self {
                    self.filter_options.$function_name = Some($function_name);

                    self
                }
            };
        }
    };
}

gen_macro!(impl_guild_id, guild_id, GuildId);
gen_macro!(impl_author_id, author_id, UserId);
gen_macro!(impl_message_id, message_id, MessageId);
gen_macro!(impl_channel_id, channel_id, ChannelId);
gen_macro!(impl_custom_ids, custom_ids, Vec<String>);

// Specific implementations of CollectorBuilder for each Collectable type
impl CollectorBuilder<'_, Event> {
    fn validate_related_ids(self) -> Result<Self> {
        let related = self.filter_options.event_types.iter().map(EventType::related_ids).fold(
            RelatedIdsForEventType::default(),
            |mut acc, e| {
                acc.user_id |= e.user_id;
                acc.guild_id |= e.guild_id;
                acc.channel_id |= e.channel_id;
                acc.message_id |= e.message_id;
                acc
            },
        );

        if (self.filter_options.user_id.is_empty() || related.user_id)
            && (self.filter_options.guild_id.is_empty() || related.guild_id)
            && (self.filter_options.channel_id.is_empty() || related.channel_id)
            && (self.filter_options.message_id.is_empty() || related.message_id)
        {
            Ok(self)
        } else {
            Err(Error::Collector(CollectorError::InvalidEventIdFilters))
        }
    }

    /// Adds an [`EventType`] that this collector will collect.
    /// If an event does not have one of these types, it won't be received.
    pub fn add_event_type(mut self, event_type: EventType) -> Self {
        self.filter_options.event_types.push(event_type);
        self
    }

    /// Sets the required user ID of an event.
    /// If an event does not have this ID, it won't be received.
    ///
    /// # Errors
    /// Errors if a relevant [`EventType`] has not been added.
    pub fn add_user_id(mut self, user_id: impl Into<UserId>) -> Result<Self> {
        self.filter_options.user_id.push(user_id.into());
        self.validate_related_ids()
    }

    /// Sets the required channel ID of an event.
    /// If an event does not have this ID, it won't be received.
    ///
    /// # Errors
    /// Errors if a relevant [`EventType`] has not been added.
    pub fn add_channel_id(mut self, channel_id: impl Into<ChannelId>) -> Result<Self> {
        self.filter_options.channel_id.push(channel_id.into());
        self.validate_related_ids()
    }

    /// Sets the required guild ID of an event.
    /// If an event does not have this ID, it won't be received.
    ///
    /// # Errors
    /// Errors if a relevant [`EventType`] has not been added.
    pub fn add_guild_id(mut self, guild_id: impl Into<GuildId>) -> Result<Self> {
        self.filter_options.guild_id.push(guild_id.into());
        self.validate_related_ids()
    }

    /// Sets the required message ID of an event.
    /// If an event does not have this ID, it won't be received.
    ///
    /// # Errors
    /// Errors if a relevant [`EventType`] has not been added.
    pub fn add_message_id(mut self, message_id: impl Into<MessageId>) -> Result<Self> {
        self.filter_options.message_id.push(message_id.into());
        self.validate_related_ids()
    }
}

impl CollectorBuilder<'_, ReactionAction> {
    /// If set to `true`, added reactions will be collected.
    ///
    /// Set to `true` by default.
    pub fn added(mut self, is_accepted: bool) -> Self {
        self.filter_options.accept_added = is_accepted;

        self
    }

    /// If set to `true`, removed reactions will be collected.
    ///
    /// Set to `false` by default.
    pub fn removed(mut self, is_accepted: bool) -> Self {
        self.filter_options.accept_removed = is_accepted;

        self
    }

    impl_channel_id!("Sets the channel on which the reaction must occur. If a reaction is not on a message with this channel ID, it won't be received.");
    impl_guild_id!("Sets the guild in which the reaction must occur. If a reaction is not on a message with this guild ID, it won't be received.");
    impl_message_id!("Sets the message on which the reaction must occur. If a reaction is not on a message with this ID, it won't be received.");
    impl_author_id!("Sets the required author ID of a reaction. If a reaction is not issued by a user with this ID, it won't be received.");
}

impl CollectorBuilder<'_, ComponentInteraction> {
    impl_channel_id!("Sets the channel on which the interaction must occur. If an interaction is not on a message with this channel ID, it won't be received.");
    impl_guild_id!("Sets the guild in which the interaction must occur. If an interaction is not on a message with this guild ID, it won't be received.");
    impl_message_id!("Sets the message on which the interaction must occur. If an interaction is not on a message with this ID, it won't be received.");
    impl_custom_ids!("Sets acceptable custom IDs for the interaction. If an interaction does not contain one of the custom IDs, it won't be received.");
    impl_author_id!("Sets the required author ID of an interaction. If an interaction is not triggered by a user with this ID, it won't be received.");
}

impl CollectorBuilder<'_, ModalSubmitInteraction> {
    impl_channel_id!("Sets the channel on which the interaction must occur. If an interaction is not on a message with this channel ID, it won't be received.");
    impl_guild_id!("Sets the guild in which the interaction must occur. If an interaction is not on a message with this guild ID, it won't be received.");
    impl_message_id!("Sets the message on which the interaction must occur. If an interaction is not on a message with this ID, it won't be received.");
    impl_custom_ids!("Sets acceptable custom IDs for the interaction. If an interaction does not contain one of the custom IDs, it won't be received.");
    impl_author_id!("Sets the required author ID of an interaction. If an interaction is not triggered by a user with this ID, it won't be received.");
}

impl CollectorBuilder<'_, Message> {
    impl_channel_id!("Sets the required channel ID of a message. If a message does not meet this ID, it won't be received.");
    impl_author_id!("Sets the required author ID of a message. If a message does not meet this ID, it won't be received.");
    impl_guild_id!("Sets the required guild ID of a message. If a message does not meet this ID, it won't be received.");
}
