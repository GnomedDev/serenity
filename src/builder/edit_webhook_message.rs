use std::collections::HashMap;

use super::CreateAllowedMentions;
use crate::builder::CreateComponents;
use crate::internal::prelude::*;
use crate::json::to_value;

/// A builder to specify the fields to edit in an existing [`Webhook`]'s message.
///
/// [`Webhook`]: crate::model::webhook::Webhook
#[derive(Clone, Debug, Default)]
pub struct EditWebhookMessage(pub HashMap<&'static str, Value>);

impl EditWebhookMessage {
    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content(&mut self, content: impl Into<String>) -> &mut Self {
        self.0.insert("content", Value::String(content.into()));
        self
    }

    /// Set the embeds associated with the message.
    ///
    /// This should be used in combination with [`Embed::fake`], creating one
    /// or more fake embeds to send to the API.
    ///
    /// # Examples
    ///
    /// Refer to [struct-level documentation of `ExecuteWebhook`] for an example
    /// on how to use embeds.
    ///
    /// [`Embed::fake`]: crate::model::channel::Embed::fake
    /// [struct-level documentation of `ExecuteWebhook`]: crate::builder::ExecuteWebhook#examples
    #[inline]
    pub fn embeds(&mut self, embeds: Vec<Value>) -> &mut Self {
        self.0.insert("embeds", Value::from(embeds));
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateAllowedMentions) -> &mut CreateAllowedMentions,
    {
        let mut allowed_mentions = CreateAllowedMentions::default();
        f(&mut allowed_mentions);
        let map = to_value(allowed_mentions).expect("AllowedMentions builder should not fail!");

        self.0.insert("allowed_mentions", map);
        self
    }

    /// Creates components for this message. Requires an application-owned webhook, meaning either
    /// the webhook's `kind` field is set to [`WebhookType::Application`], or it was created by an
    /// application (and has kind [`WebhookType::Incoming`]).
    ///
    /// [`WebhookType::Application`]: crate::model::webhook::WebhookType
    /// [`WebhookType::Incoming`]: crate::model::webhook::WebhookType
    pub fn components<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateComponents) -> &mut CreateComponents,
    {
        let mut components = CreateComponents::default();
        f(&mut components);
        let map = to_value(components).expect("CreateComponents builder should not fail!");

        self.0.insert("components", map);
        self
    }
}
