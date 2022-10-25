use std::sync::Arc;

use super::ReactionAction;
use crate::model::channel::Reaction;

/// A trait to generalise over LazyArc and LazyReactionAction
pub trait LazyItem<Item: ?Sized> {
    fn as_arc(&mut self) -> &mut Arc<Item>;
}

/// Wraps a `&T` and clones the value into an [`Arc<T>`] lazily. Used with collectors to allow inspecting
/// the value in filters while only cloning values that actually match.
#[derive(Debug)]
pub struct LazyArc<'a, T> {
    value: &'a T,
    arc: Option<Arc<T>>,
}

impl<'a, T: Clone> LazyArc<'a, T> {
    pub fn new(value: &'a T) -> Self {
        LazyArc {
            value,
            arc: None,
        }
    }
}

impl<Item: Clone> LazyItem<Item> for LazyArc<'_, Item> {
    fn as_arc(&mut self) -> &mut Arc<Item> {
        let value = self.value;
        self.arc.get_or_insert_with(|| Arc::new(value.clone()))
    }
}

impl<'a, T> std::ops::Deref for LazyArc<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

#[derive(Debug)]
pub struct LazyReactionAction<'a> {
    pub(super) reaction: LazyArc<'a, Reaction>,
    arc: Option<Arc<ReactionAction>>,
    pub(super) added: bool,
}

impl<'a> LazyReactionAction<'a> {
    #[must_use]
    pub fn new(reaction: &'a Reaction, added: bool) -> Self {
        Self {
            reaction: LazyArc::new(reaction),
            added,
            arc: None,
        }
    }
}

impl LazyItem<ReactionAction> for LazyReactionAction<'_> {
    fn as_arc(&mut self) -> &mut Arc<ReactionAction> {
        let added = self.added;
        let reaction = &mut self.reaction;
        self.arc.get_or_insert_with(|| {
            Arc::new(if added {
                ReactionAction::Added(reaction.as_arc().clone())
            } else {
                ReactionAction::Removed(reaction.as_arc().clone())
            })
        })
    }
}
