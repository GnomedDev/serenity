use std::sync::Arc;

use tokio::sync::mpsc::{
    unbounded_channel,
    UnboundedReceiver as Receiver,
    UnboundedSender as Sender,
};

use super::filter_options::CommonFilterOptions;
use super::lazy_item::{LazyArc, LazyItem, LazyReactionAction};
use super::{Collectable, ReactionAction};
use crate::model::channel::Message;
use crate::model::event::Event;
use crate::model::prelude::{ComponentInteraction, ModalSubmitInteraction};

pub trait FilterTrait<Item: Collectable> {
    fn is_passing_constraints(&self, item: &mut Item::Lazy<'_>) -> bool;
}

#[derive(Clone, Debug)]
pub struct Filter<Item: Collectable> {
    pub(super) filtered: u32,
    pub(super) collected: u32,
    pub(crate) sender: Sender<Arc<Item>>,
    pub(super) options: Item::FilterOptions,
    pub(super) common_options: CommonFilterOptions<Item::FilterItem>,
}

impl<Item: Collectable> Filter<Item> {
    /// Creates a new filter
    pub fn new(
        options: Item::FilterOptions,
        common_options: CommonFilterOptions<Item::FilterItem>,
    ) -> (Self, Receiver<Arc<Item>>) {
        let (sender, receiver) = unbounded_channel();

        let filter = Self {
            filtered: 0,
            collected: 0,
            sender,
            options,
            common_options,
        };

        (filter, receiver)
    }

    fn is_within_limits(&self) -> bool {
        self.common_options.filter_limit.map_or(true, |limit| self.filtered < limit.get())
            && self.common_options.collect_limit.map_or(true, |limit| self.collected < limit.get())
    }
}

impl<Item: Collectable> Filter<Item>
where
    Filter<Item>: FilterTrait<Item>,
{
    /// Sends an item to the consuming collector if the item conforms
    /// to the constraints and the limits are not reached yet.
    pub(crate) fn process_item(&mut self, mut item: Item::Lazy<'_>) -> bool {
        if self.is_passing_constraints(&mut item) {
            self.collected += 1;

            if self.sender.send(item.as_arc().clone()).is_err() {
                return false;
            }
        }

        self.filtered += 1;

        self.is_within_limits() && !self.sender.is_closed()
    }
}

impl FilterTrait<ReactionAction> for Filter<ReactionAction> {
    /// Checks if the `reaction` passes set constraints.
    /// Constraints are optional, as it is possible to limit reactions to
    /// be sent by a specific author or in a specific guild.
    fn is_passing_constraints(&self, reaction: &mut LazyReactionAction<'_>) -> bool {
        let reaction = match (reaction.added, &reaction.reaction) {
            (true, reaction) => {
                if self.options.accept_added {
                    reaction
                } else {
                    return false;
                }
            },
            (false, reaction) => {
                if self.options.accept_removed {
                    reaction
                } else {
                    return false;
                }
            },
        };

        self.options.guild_id.map_or(true, |id| Some(id) == reaction.guild_id)
            && self.options.message_id.map_or(true, |id| id == reaction.message_id)
            && self.options.channel_id.map_or(true, |id| id == reaction.channel_id)
            && self.options.author_id.map_or(true, |id| Some(id) == reaction.user_id)
            && self.common_options.filter.as_ref().map_or(true, |f| f.0(reaction))
    }
}

impl FilterTrait<ComponentInteraction> for Filter<ComponentInteraction> {
    /// Checks if the `interaction` passes set constraints.
    /// Constraints are optional, as it is possible to limit interactions to
    /// be sent by a specific author or in a specific guild.
    fn is_passing_constraints(&self, interaction: &mut LazyArc<'_, ComponentInteraction>) -> bool {
        self.options.guild_id.map_or(true, |id| Some(id) == interaction.guild_id)
            && self
                .options
                .custom_ids
                .as_ref()
                .map_or(true, |id| id.contains(&interaction.data.custom_id))
            && self.options.message_id.map_or(true, |id| interaction.message.id == id)
            && self.options.channel_id.map_or(true, |id| id == interaction.channel_id)
            && self.options.author_id.map_or(true, |id| id == interaction.user.id)
            && self.common_options.filter.as_ref().map_or(true, |f| f.0(interaction))
    }
}

impl FilterTrait<Event> for Filter<Event> {
    /// Checks if the `event` passes set constraints.
    /// Constraints are optional, as it is possible to limit events to
    /// be sent by a specific user or in a specific guild.
    fn is_passing_constraints(&self, event: &mut LazyArc<'_, Event>) -> bool {
        fn empty_or_any<T, F>(slice: &[T], f: F) -> bool
        where
            F: Fn(&T) -> bool,
        {
            slice.is_empty() || slice.iter().any(f)
        }

        self.options.event_types.contains(&event.event_type())
            && empty_or_any(&self.options.guild_id, |id| event.guild_id().contains(id))
            && empty_or_any(&self.options.user_id, |id| event.user_id().contains(id))
            && empty_or_any(&self.options.channel_id, |id| event.channel_id().contains(id))
            && empty_or_any(&self.options.message_id, |id| event.message_id().contains(id))
            && self.common_options.filter.as_ref().map_or(true, |f| f.0(event))
    }
}

impl FilterTrait<Message> for Filter<Message> {
    /// Checks if the `message` passes set constraints.
    /// Constraints are optional, as it is possible to limit messages to
    /// be sent by a specific author or in a specific guild.
    fn is_passing_constraints(&self, message: &mut LazyArc<'_, Message>) -> bool {
        self.options.guild_id.map_or(true, |g| Some(g) == message.guild_id)
            && self.options.channel_id.map_or(true, |g| g == message.channel_id)
            && self.options.author_id.map_or(true, |g| g == message.author.id)
            && self.common_options.filter.as_ref().map_or(true, |f| f.0(message))
    }
}

impl FilterTrait<ModalSubmitInteraction> for Filter<ModalSubmitInteraction> {
    fn is_passing_constraints(
        &self,
        interaction: &mut LazyArc<'_, ModalSubmitInteraction>,
    ) -> bool {
        self.options.guild_id.map_or(true, |id| Some(id) == interaction.guild_id)
            && self
                .options
                .custom_ids
                .as_ref()
                .map_or(true, |id| id.contains(&interaction.data.custom_id))
            && self
                .options
                .message_id
                .map_or(true, |id| Some(id) == interaction.message.as_ref().map(|m| m.id))
            && self.options.channel_id.map_or(true, |id| id == interaction.channel_id)
            && self.options.author_id.map_or(true, |id| id == interaction.user.id)
            && self.common_options.filter.as_ref().map_or(true, |f| f.0(interaction))
    }
}
