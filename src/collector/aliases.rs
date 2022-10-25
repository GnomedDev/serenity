use super::{CollectorBuilder, ReactionAction};
use crate::model::channel::Message;
use crate::model::event::Event;
use crate::model::prelude::{ComponentInteraction, ModalSubmitInteraction};

macro_rules! builder_alias {
    ($name:ident, $target:ident) => {
        pub type $name<'a> = CollectorBuilder<'a, $target>;
    };
}

builder_alias!(EventCollectorBuilder, Event);
builder_alias!(MessageCollectorBuilder, Message);
builder_alias!(ReactionCollectorBuilder, ReactionAction);
builder_alias!(ModalInteractionCollectorBuilder, ModalSubmitInteraction);
builder_alias!(ComponentInteractionCollectorBuilder, ComponentInteraction);
