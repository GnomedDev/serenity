use super::{CreateAllowedMentions, CreateComponents, CreateEmbed};
use crate::model::channel::{AttachmentType, MessageFlags};
use crate::model::id::AttachmentId;

/// A builder to specify the fields to edit in an existing message.
///
/// # Examples
///
/// Editing the content of a [`Message`] to `"hello"`:
///
/// ```rust,no_run
/// # use serenity::model::id::{ChannelId, MessageId};
/// # #[cfg(feature = "client")]
/// # use serenity::client::Context;
/// # #[cfg(feature = "framework")]
/// # use serenity::framework::standard::{CommandResult, macros::command};
/// #
/// # #[cfg(all(feature = "model", feature = "utils", feature = "framework"))]
/// # #[command]
/// # async fn example(ctx: &Context) -> CommandResult {
/// # let mut message = ChannelId::new(7).message(&ctx, MessageId::new(8)).await?;
/// message.edit(ctx, |m| m.content("hello")).await?;
/// # Ok(())
/// # }
/// ```
///
/// [`Message`]: crate::model::channel::Message
#[derive(Clone, Debug, Default, Serialize)]
pub struct EditMessage<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<Vec<CreateEmbed>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<MessageFlags>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<CreateAllowedMentions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<CreateComponents>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attachments: Option<Vec<AttachmentId>>,

    #[serde(skip)]
    pub(crate) files: Vec<AttachmentType<'a>>,
}

impl<'a> EditMessage<'a> {
    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content(&mut self, content: impl Into<String>) -> &mut Self {
        self.content = Some(content.into());
        self
    }

    fn embeds(&mut self) -> &mut Vec<CreateEmbed> {
        self.embeds.get_or_insert_with(Vec::new)
    }

    fn _add_embed(&mut self, embed: CreateEmbed) -> &mut Self {
        self.embeds().push(embed);

        self
    }

    /// Add an embed for the message.
    ///
    /// **Note**: This will keep all existing embeds. Use [`Self::set_embed()`] to replace existing
    /// embeds.
    pub fn add_embed<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
    {
        let mut embed = CreateEmbed::default();
        f(&mut embed);
        self._add_embed(embed)
    }

    /// Add multiple embeds for the message.
    ///
    /// **Note**: This will keep all existing embeds. Use [`Self::set_embeds()`] to replace existing
    /// embeds.
    pub fn add_embeds(&mut self, embeds: Vec<CreateEmbed>) -> &mut Self {
        self.embeds().extend(embeds);
        self
    }

    /// Set an embed for the message.
    ///
    /// Equivalent to [`Self::set_embed()`].
    ///
    /// **Note**: This will replace all existing embeds. Use
    /// [`Self::add_embed()`] to add an additional embed.
    pub fn embed<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
    {
        let mut embed = CreateEmbed::default();
        f(&mut embed);
        self.set_embed(embed)
    }

    /// Set an embed for the message.
    ///
    /// Equivalent to [`Self::embed()`].
    ///
    /// **Note**: This will replace all existing embeds.
    /// Use [`Self::add_embed()`] to add an additional embed.
    pub fn set_embed(&mut self, embed: CreateEmbed) -> &mut Self {
        self.set_embeds(vec![embed])
    }

    /// Set multiple embeds for the message.
    ///
    /// **Note**: This will replace all existing embeds. Use [`Self::add_embeds()`] to keep existing
    /// embeds.
    pub fn set_embeds(&mut self, embeds: Vec<CreateEmbed>) -> &mut Self {
        self.embeds = Some(embeds);
        self
    }

    /// Suppress or unsuppress embeds in the message, this includes those generated by Discord
    /// themselves.
    pub fn suppress_embeds(&mut self, suppress: bool) -> &mut Self {
        // `1 << 2` is defined by the API to be the SUPPRESS_EMBEDS flag.
        // At the time of writing, the only accepted value in "flags" is `SUPPRESS_EMBEDS` for editing messages.
        let flags =
            suppress.then(|| MessageFlags::SUPPRESS_EMBEDS).unwrap_or_else(MessageFlags::empty);

        self.flags = Some(flags);
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateAllowedMentions) -> &mut CreateAllowedMentions,
    {
        let mut allowed_mentions = CreateAllowedMentions::default();
        f(&mut allowed_mentions);

        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Creates components for this message.
    pub fn components<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateComponents) -> &mut CreateComponents,
    {
        let mut components = CreateComponents::default();
        f(&mut components);

        self.set_components(components)
    }

    /// Sets the components of this message.
    pub fn set_components(&mut self, components: CreateComponents) -> &mut Self {
        self.components = Some(components);
        self
    }

    /// Sets the flags for the message.
    pub fn flags(&mut self, flags: MessageFlags) -> &mut Self {
        self.flags = Some(flags);
        self
    }

    /// Add a new attachment for the message.
    ///
    /// This can be called multiple times.
    pub fn attachment(&mut self, attachment: impl Into<AttachmentType<'a>>) -> &mut Self {
        self.files.push(attachment.into());
        self
    }

    fn attachments(&mut self) -> &mut Vec<AttachmentId> {
        self.attachments.get_or_insert_with(Vec::new)
    }

    /// Add an existing attachment by id.
    pub fn add_existing_attachment(&mut self, attachment: AttachmentId) -> &mut Self {
        self.attachments().push(attachment);
        self
    }

    /// Remove an existing attachment by id.
    pub fn remove_existing_attachment(&mut self, attachment: AttachmentId) -> &mut Self {
        if let Some(attachments) = &mut self.attachments {
            if let Some(attachment_index) = attachments.iter().position(|a| *a == attachment) {
                attachments.remove(attachment_index);
            };
        }

        self
    }
}
