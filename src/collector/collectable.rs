use super::filter_options::{
    ComponentFilterOptions,
    EventFilterOptions,
    MessageFilterOptions,
    ModalFilterOptions,
    ReactionFilterOptions,
};
use super::lazy_item::{LazyArc, LazyItem, LazyReactionAction};
use super::sealed::Sealed;
use super::ReactionAction;
use crate::model::channel::{Message, Reaction};
use crate::model::event::Event;
use crate::model::prelude::{ComponentInteraction, Interaction, ModalSubmitInteraction};

pub trait Collectable: Sealed + Sized {
    type Lazy<'a>: LazyItem<Self>;
    type FilterOptions: Default;
    type FilterItem;

    fn extract(event: &mut Event) -> Option<Self::Lazy<'_>>;
}

impl super::Collectable for ModalSubmitInteraction {
    type FilterOptions = ModalFilterOptions;
    type FilterItem = ModalSubmitInteraction;
    type Lazy<'a> = LazyArc<'a, ModalSubmitInteraction>;

    fn extract(event: &mut Event) -> Option<Self::Lazy<'_>> {
        if let Event::InteractionCreate(interaction) = event {
            if let Interaction::ModalSubmit(interaction) = &mut interaction.interaction {
                return Some(LazyArc::new(interaction));
            }
        };

        None
    }
}

impl super::Collectable for ReactionAction {
    type FilterItem = Reaction;
    type FilterOptions = ReactionFilterOptions;
    type Lazy<'a> = LazyReactionAction<'a>;

    fn extract(event: &mut Event) -> Option<Self::Lazy<'_>> {
        match event {
            Event::ReactionAdd(reaction) => Some(LazyReactionAction::new(&reaction.reaction, true)),
            Event::ReactionRemove(reaction) => {
                Some(LazyReactionAction::new(&reaction.reaction, false))
            },
            _ => None,
        }
    }
}

impl super::Collectable for ComponentInteraction {
    type FilterOptions = ComponentFilterOptions;
    type FilterItem = ComponentInteraction;
    type Lazy<'a> = LazyArc<'a, ComponentInteraction>;

    fn extract(item: &mut Event) -> Option<Self::Lazy<'_>> {
        if let Event::InteractionCreate(interaction) = item {
            if let Interaction::Component(interaction) = &mut interaction.interaction {
                return Some(LazyArc::new(interaction));
            }
        };

        None
    }
}

impl super::Collectable for Message {
    type FilterItem = Message;
    type FilterOptions = MessageFilterOptions;
    type Lazy<'a> = LazyArc<'a, Message>;

    fn extract(event: &mut Event) -> Option<Self::Lazy<'_>> {
        if let Event::MessageCreate(message) = event {
            Some(LazyArc::new(&message.message))
        } else {
            None
        }
    }
}

impl super::Collectable for Event {
    type FilterItem = Event;
    type FilterOptions = EventFilterOptions;
    type Lazy<'a> = LazyArc<'a, Event>;

    fn extract(item: &mut Event) -> Option<Self::Lazy<'_>> {
        Some(LazyArc::new(item))
    }
}
