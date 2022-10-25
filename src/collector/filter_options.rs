use std::num::NonZeroU32;

use derivative::Derivative;

use super::FilterFn;
use crate::model::event::EventType;
use crate::model::id::{ChannelId, GuildId, MessageId, UserId};

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""), Default(bound = ""))]
pub struct CommonFilterOptions<FilterItem> {
    pub(super) filter_limit: Option<NonZeroU32>,
    pub(super) collect_limit: Option<NonZeroU32>,
    pub(super) filter: Option<FilterFn<FilterItem>>,
}

#[derive(Clone, Debug, Default)]
pub struct ComponentFilterOptions {
    pub(super) channel_id: Option<ChannelId>,
    pub(super) guild_id: Option<GuildId>,
    pub(super) author_id: Option<UserId>,
    pub(super) message_id: Option<MessageId>,
    pub(super) custom_ids: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default)]
pub struct MessageFilterOptions {
    pub(super) channel_id: Option<ChannelId>,
    pub(super) guild_id: Option<GuildId>,
    pub(super) author_id: Option<UserId>,
}

#[derive(Clone, Debug, Default)]
pub struct ModalFilterOptions {
    pub(super) channel_id: Option<ChannelId>,
    pub(super) guild_id: Option<GuildId>,
    pub(super) author_id: Option<UserId>,
    pub(super) message_id: Option<MessageId>,
    pub(super) custom_ids: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default)]
pub struct EventFilterOptions {
    pub(super) event_types: Vec<EventType>,
    pub(super) channel_id: Vec<ChannelId>,
    pub(super) guild_id: Vec<GuildId>,
    pub(super) user_id: Vec<UserId>,
    pub(super) message_id: Vec<MessageId>,
}

#[derive(Clone, Debug)]
pub struct ReactionFilterOptions {
    pub(super) channel_id: Option<ChannelId>,
    pub(super) guild_id: Option<GuildId>,
    pub(super) author_id: Option<UserId>,
    pub(super) message_id: Option<MessageId>,
    pub(super) accept_added: bool,
    pub(super) accept_removed: bool,
}

impl Default for ReactionFilterOptions {
    fn default() -> Self {
        Self {
            channel_id: None,
            guild_id: None,
            author_id: None,
            message_id: None,
            accept_added: true,
            accept_removed: false,
        }
    }
}
