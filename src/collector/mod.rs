//! Collectors will receive events from the contextual shard, check if the
//! filter lets them pass, and collects if the receive, collect, or time limits
//! are not reached yet.

// triggered by Derivative
// can't put it on the derived type itself for some reason
#![allow(clippy::let_underscore_must_use)]

use std::fmt;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context as FutContext, Poll};

use derivative::Derivative;
use futures::Future;
use tokio::sync::mpsc::UnboundedReceiver as Receiver;
use tokio::time::Sleep;

use crate::model::channel::Reaction;
use crate::model::event::Event;

mod error;
pub use error::Error as CollectorError;

mod aliases;
mod collectable;
mod collector_builder;
mod filter;
mod filter_options;
mod lazy_item;

pub use aliases::*;
use collectable::Collectable;
pub use collector_builder::CollectorBuilder;
pub use filter::Filter;

type FilterFnInner<Arg> = dyn Fn(&Arg) -> bool + 'static + Send + Sync;

pub struct CollectorCallback(pub Box<dyn FnMut(&mut Event) -> bool + Send + Sync>);

pub struct Collector<Item> {
    pub(super) receiver: Pin<Box<Receiver<Arc<Item>>>>,
    pub(super) timeout: Option<Pin<Box<Sleep>>>,
}

impl<Item> Collector<Item> {
    /// Stops collecting, this will implicitly be done once the
    /// collector drops.
    /// In case the drop does not appear until later, it is preferred to
    /// stop the collector early.
    pub fn stop(self) {}
}

impl<Item> futures::stream::Stream for Collector<Item> {
    type Item = Arc<Item>;

    fn poll_next(mut self: Pin<&mut Self>, ctx: &mut FutContext<'_>) -> Poll<Option<Self::Item>> {
        if let Some(timeout) = &mut self.timeout {
            match timeout.as_mut().poll(ctx) {
                Poll::Ready(_) => {
                    return Poll::Ready(None);
                },
                Poll::Pending => (),
            }
        }

        self.receiver.as_mut().poll_recv(ctx)
    }
}

impl std::fmt::Debug for CollectorCallback {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CollectorCallback").finish()
    }
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
pub struct FilterFn<Arg: ?Sized>(Arc<FilterFnInner<Arg>>);

impl<Arg> fmt::Debug for FilterFn<Arg> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("FilterFn")
            .field(&format_args!("Arc<dyn Fn({}) -> bool", stringify!(Arg)))
            .finish()
    }
}

mod sealed {
    use crate::model::prelude::*;

    pub trait Sealed {}

    impl Sealed for Event {}
    impl Sealed for Message {}
    impl Sealed for crate::collector::ReactionAction {}
    impl Sealed for interaction::modal::ModalSubmitInteraction {}
    impl Sealed for interaction::message_component::ComponentInteraction {}
}

/// Marks whether the reaction has been added or removed.
#[derive(Debug, Clone)]
pub enum ReactionAction {
    Added(Arc<Reaction>),
    Removed(Arc<Reaction>),
}

impl ReactionAction {
    #[must_use]
    pub fn as_inner_ref(&self) -> &Arc<Reaction> {
        match self {
            Self::Added(inner) | Self::Removed(inner) => inner,
        }
    }

    #[must_use]
    pub fn is_added(&self) -> bool {
        matches!(self, Self::Added(_))
    }

    #[must_use]
    pub fn is_removed(&self) -> bool {
        matches!(self, Self::Removed(_))
    }
}
