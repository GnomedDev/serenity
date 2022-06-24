#[cfg(feature = "model")]
use std::fmt::Write as _;
#[cfg(feature = "model")]
use std::sync::Arc;

#[cfg(feature = "model")]
use futures::stream::Stream;

#[cfg(feature = "model")]
use crate::builder::{
    CreateInvite,
    CreateMessage,
    CreateStageInstance,
    CreateThread,
    CreateWebhook,
    EditChannel,
    EditMessage,
    EditStageInstance,
    EditThread,
    GetMessages,
    SearchFilter,
};
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::Cache;
#[cfg(feature = "collector")]
use crate::client::bridge::gateway::ShardMessenger;
#[cfg(feature = "collector")]
use crate::collector::{MessageCollectorBuilder, ReactionCollectorBuilder};
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http, Typing};
#[cfg(feature = "model")]
use crate::json::json;
#[cfg(feature = "model")]
use crate::model::channel::AttachmentType;
use crate::model::prelude::*;

#[cfg(feature = "model")]
impl ChannelId {
    /// Broadcasts that the current user is typing to a channel for the next 5
    /// seconds.
    ///
    /// After 5 seconds, another request must be made to continue broadcasting
    /// that the current user is typing.
    ///
    /// This should rarely be used for bots, and should likely only be used for
    /// signifying that a long-running command is still being executed.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use serenity::model::id::ChannelId;
    ///
    /// # async fn run() {
    /// # let http = serenity::http::Http::new("token");
    /// let _successful = ChannelId::new(7).broadcast_typing(&http).await;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks
    /// permission to send messages to this channel.
    ///
    /// [Send Messages]: Permissions::SEND_MESSAGES
    #[inline]
    pub async fn broadcast_typing(self, http: impl AsRef<Http>) -> Result<()> {
        http.as_ref().broadcast_typing(self.get()).await
    }

    /// Creates an invite leading to the given channel.
    ///
    /// **Note**: Requires the [Create Instant Invite] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Create Instant Invite]: Permissions::CREATE_INSTANT_INVITE
    pub async fn create_invite<F>(&self, http: impl AsRef<Http>, f: F) -> Result<RichInvite>
    where
        F: FnOnce(&mut CreateInvite) -> &mut CreateInvite,
    {
        let mut invite = CreateInvite::default();
        f(&mut invite);

        http.as_ref().create_invite(self.get(), &invite, None).await
    }

    /// Creates a [permission overwrite][`PermissionOverwrite`] for either a
    /// single [`Member`] or [`Role`] within the channel.
    ///
    /// Refer to the documentation for [`GuildChannel::create_permission`] for
    /// more information.
    ///
    /// Requires the [Manage Channels] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if an invalid value is set.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    pub async fn create_permission(
        self,
        http: impl AsRef<Http>,
        target: PermissionOverwrite,
    ) -> Result<()> {
        let data: PermissionOverwriteData = target.into();
        http.as_ref().create_permission(self.get(), data.id.get(), &data).await
    }

    /// React to a [`Message`] with a custom [`Emoji`] or unicode character.
    ///
    /// [`Message::react`] may be a more suited method of reacting in most
    /// cases.
    ///
    /// Requires the [Add Reactions] permission, _if_ the current user is the
    /// first user to perform a react with a certain emoji.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Add Reactions]: Permissions::ADD_REACTIONS
    #[inline]
    pub async fn create_reaction(
        self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()> {
        http.as_ref()
            .create_reaction(self.get(), message_id.into().get(), &reaction_type.into())
            .await
    }

    /// Deletes this channel, returning the channel on a successful deletion.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    #[inline]
    pub async fn delete(self, http: impl AsRef<Http>) -> Result<Channel> {
        http.as_ref().delete_channel(self.get()).await
    }

    /// Deletes a [`Message`] given its Id.
    ///
    /// Refer to [`Message::delete`] for more information.
    ///
    /// Requires the [Manage Messages] permission, if the current user is not
    /// the author of the message.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission to
    /// delete the message.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    #[inline]
    pub async fn delete_message(
        self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
    ) -> Result<()> {
        http.as_ref().delete_message(self.get(), message_id.into().get()).await
    }

    /// Deletes all messages by Ids from the given vector in the given channel.
    ///
    /// The minimum amount of messages is 2 and the maximum amount is 100.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// **Note**: Messages that are older than 2 weeks can't be deleted using
    /// this method.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::BulkDeleteAmount`] if an attempt was made to
    /// delete either 0 or more than 100 messages.
    ///
    /// Also will return [`Error::Http`] if the current user lacks permission
    /// to delete messages.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    pub async fn delete_messages<T, It>(self, http: impl AsRef<Http>, message_ids: It) -> Result<()>
    where
        T: AsRef<MessageId>,
        It: IntoIterator<Item = T>,
    {
        let ids =
            message_ids.into_iter().map(|message_id| message_id.as_ref().0).collect::<Vec<_>>();

        let len = ids.len();

        if len == 0 || len > 100 {
            return Err(Error::Model(ModelError::BulkDeleteAmount));
        }

        if ids.len() == 1 {
            self.delete_message(http, ids[0]).await
        } else {
            let map = json!({ "messages": ids });

            http.as_ref().delete_messages(self.get(), &map).await
        }
    }

    /// Deletes all permission overrides in the channel from a member or role.
    ///
    /// **Note**: Requires the [Manage Channel] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Channel]: Permissions::MANAGE_CHANNELS
    pub async fn delete_permission(
        self,
        http: impl AsRef<Http>,
        permission_type: PermissionOverwriteType,
    ) -> Result<()> {
        http.as_ref()
            .delete_permission(self.get(), match permission_type {
                PermissionOverwriteType::Member(id) => id.get(),
                PermissionOverwriteType::Role(id) => id.get(),
            })
            .await
    }

    /// Deletes the given [`Reaction`] from the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current
    /// user did not perform the reaction.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user did not perform the reaction,
    /// and lacks permission.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    #[inline]
    pub async fn delete_reaction(
        self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        user_id: Option<UserId>,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()> {
        http.as_ref()
            .delete_reaction(
                self.get(),
                message_id.into().get(),
                user_id.map(UserId::get),
                &reaction_type.into(),
            )
            .await
    }

    /// Deletes all [`Reaction`]s of the given emoji to a message within the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    #[inline]
    pub async fn delete_reaction_emoji(
        self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()> {
        http.as_ref()
            .delete_message_reaction_emoji(
                self.get(),
                message_id.into().get(),
                &reaction_type.into(),
            )
            .await
    }

    /// Edits the settings of a [`Channel`], optionally setting new values.
    ///
    /// Refer to [`EditChannel`]'s documentation for its methods.
    ///
    /// Requires the [Manage Channel] permission.
    ///
    /// # Examples
    ///
    /// Change a voice channel's name and bitrate:
    ///
    /// ```rust,no_run
    /// // assuming a `channel_id` has been bound
    ///
    /// # async fn run() {
    /// #     use serenity::http::Http;
    /// #     use serenity::model::id::ChannelId;
    /// #     let http = Http::new("token");
    /// #     let channel_id = ChannelId::new(1234);
    /// channel_id.edit(&http, |c| c.name("test").bitrate(64000)).await;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if an invalid value is set.
    ///
    /// [Manage Channel]: Permissions::MANAGE_CHANNELS
    #[inline]
    pub async fn edit<F>(self, http: impl AsRef<Http>, f: F) -> Result<GuildChannel>
    where
        F: FnOnce(&mut EditChannel) -> &mut EditChannel,
    {
        let mut channel = EditChannel::default();
        f(&mut channel);

        http.as_ref().edit_channel(self.get(), &channel, None).await
    }

    /// Edits a [`Message`] in the channel given its Id.
    ///
    /// Message editing preserves all unchanged message data.
    ///
    /// Refer to the documentation for [`EditMessage`] for more information
    /// regarding message restrictions and requirements.
    ///
    /// **Note**: Requires that the current user be the author of the message.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over the [`the limit`], containing the number of unicode code points
    /// over the limit.
    ///
    /// [`the limit`]: crate::builder::EditMessage::content
    #[inline]
    pub async fn edit_message<'a, F>(
        self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        f: F,
    ) -> Result<Message>
    where
        F: for<'b> FnOnce(&'b mut EditMessage<'a>) -> &'b mut EditMessage<'a>,
    {
        let mut msg = EditMessage::default();
        f(&mut msg);

        if let Some(content) = &msg.content {
            if let Some(length_over) = Message::overflow_length(content) {
                return Err(Error::Model(ModelError::MessageTooLong(length_over)));
            }
        }

        let files = std::mem::take(&mut msg.files);
        http.as_ref()
            .edit_message_and_attachments(self.get(), message_id.into().get(), &msg, files)
            .await
    }

    /// Follows the News Channel
    ///
    /// Requires [Manage Webhook] permissions on the target channel.
    ///
    /// **Note**: Only available on news channels.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    /// [Manage Webhook]: Permissions::MANAGE_WEBHOOKS
    pub async fn follow(
        self,
        http: impl AsRef<Http>,
        target_channel_id: impl Into<ChannelId>,
    ) -> Result<FollowedChannel> {
        http.as_ref().follow_news_channel(self.get(), target_channel_id.into().get()).await
    }

    /// Attempts to find a [`Channel`] by its Id in the cache.
    #[cfg(feature = "cache")]
    #[inline]
    pub fn to_channel_cached(self, cache: impl AsRef<Cache>) -> Option<Channel> {
        cache.as_ref().channel(self)
    }

    /// First attempts to find a [`Channel`] by its Id in the cache,
    /// upon failure requests it via the REST API.
    ///
    /// **Note**: If the `cache`-feature is enabled permissions will be checked and upon owning the
    /// required permissions the HTTP-request will be issued. Additionally, you might want to
    /// enable the `temp_cache` feature to cache channel data retrieved by this function for a
    /// short duration.
    #[allow(clippy::missing_errors_doc)]
    #[inline]
    pub async fn to_channel(self, cache_http: impl CacheHttp) -> Result<Channel> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(channel) = cache.channel(self) {
                    return Ok(channel);
                }
            }
        }

        let channel = cache_http.http().get_channel(self.get()).await?;

        #[cfg(all(feature = "cache", feature = "temp_cache"))]
        {
            if let Some(cache) = cache_http.cache() {
                if let Channel::Guild(guild_channel) = &channel {
                    cache.temp_channels.insert(guild_channel.id, guild_channel.clone());
                }
            }
        }

        Ok(channel)
    }

    /// Gets all of the channel's invites.
    ///
    /// Requires the [Manage Channels] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    #[inline]
    pub async fn invites(self, http: impl AsRef<Http>) -> Result<Vec<RichInvite>> {
        http.as_ref().get_channel_invites(self.get()).await
    }

    /// Gets a message from the channel.
    ///
    /// If the cache feature is enabled the cache will be checked
    /// first. If not found it will resort to an http request.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    #[inline]
    pub async fn message(
        self,
        cache_http: impl CacheHttp,
        message_id: impl Into<MessageId>,
    ) -> Result<Message> {
        let message_id = message_id.into();

        #[cfg(feature = "cache")]
        if let Some(cache) = cache_http.cache() {
            if let Some(message) = cache.message(self, message_id) {
                return Ok(message);
            }
        }

        cache_http.http().get_message(self.get(), message_id.get()).await
    }

    /// Gets messages from the channel.
    ///
    /// Refer to [`GetMessages`] for more information on how to use `builder`.
    ///
    /// **Note**: Returns an empty [`Vec`] if the current user
    /// does not have the [Read Message History] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user does not have
    /// permission to view the channel.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    pub async fn messages<F>(self, http: impl AsRef<Http>, builder: F) -> Result<Vec<Message>>
    where
        F: FnOnce(&mut GetMessages) -> &mut GetMessages,
    {
        let mut get_messages = GetMessages::default();
        builder(&mut get_messages);
        let mut query = format!("?limit={}", get_messages.limit.unwrap_or(50));

        if let Some(filter) = get_messages.search_filter {
            match filter {
                SearchFilter::After(after) => write!(query, "&after={}", after)?,
                SearchFilter::Around(around) => write!(query, "&around={}", around)?,
                SearchFilter::Before(before) => write!(query, "&before={}", before)?,
            }
        }

        http.as_ref().get_messages(self.get(), &query).await
    }

    /// Streams over all the messages in a channel.
    ///
    /// This is accomplished and equivalent to repeated calls to [`Self::messages`].
    /// A buffer of at most 100 messages is used to reduce the number of calls.
    /// necessary.
    ///
    /// The stream returns the newest message first, followed by older messages.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use serenity::model::id::ChannelId;
    /// # use serenity::http::Http;
    /// #
    /// # async fn run() {
    /// # let channel_id = ChannelId::new(1);
    /// # let ctx = Http::new("token");
    /// use serenity::futures::StreamExt;
    /// use serenity::model::channel::MessagesIter;
    ///
    /// let mut messages = channel_id.messages_iter(&ctx).boxed();
    /// while let Some(message_result) = messages.next().await {
    ///     match message_result {
    ///         Ok(message) => println!("{} said \"{}\".", message.author.name, message.content,),
    ///         Err(error) => eprintln!("Uh oh! Error: {}", error),
    ///     }
    /// }
    /// # }
    /// ```
    pub fn messages_iter<H: AsRef<Http>>(self, http: H) -> impl Stream<Item = Result<Message>> {
        MessagesIter::<H>::stream(http, self)
    }

    /// Returns the name of whatever channel this id holds.
    #[cfg(feature = "cache")]
    pub async fn name(self, cache: impl AsRef<Cache>) -> Option<String> {
        let channel = self.to_channel_cached(cache)?;

        Some(match channel {
            Channel::Guild(channel) => channel.name().to_string(),
            Channel::Category(category) => category.name().to_string(),
            Channel::Private(channel) => channel.name(),
        })
    }

    /// Pins a [`Message`] to the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if the channel has too many pinned messages.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    #[inline]
    pub async fn pin(self, http: impl AsRef<Http>, message_id: impl Into<MessageId>) -> Result<()> {
        http.as_ref().pin_message(self.get(), message_id.into().get(), None).await
    }

    /// Crossposts a [`Message`].
    ///
    /// Requires either to be the message author or to have manage [Manage Messages] permissions on this channel.
    ///
    /// **Note**: Only available on news channels.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// and if the user is not the author of the message.
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    pub async fn crosspost(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
    ) -> Result<Message> {
        http.as_ref().crosspost_message(self.get(), message_id.into().get()).await
    }

    /// Gets the list of [`Message`]s which are pinned to the channel.
    ///
    /// **Note**: Returns an empty [`Vec`] if the current user does not
    /// have the [Read Message History] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission
    /// to view the channel.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    #[inline]
    pub async fn pins(self, http: impl AsRef<Http>) -> Result<Vec<Message>> {
        http.as_ref().get_pins(self.get()).await
    }

    /// Gets the list of [`User`]s who have reacted to a [`Message`] with a
    /// certain [`Emoji`].
    ///
    /// The default `limit` is `50` - specify otherwise to receive a different
    /// maximum number of users. The maximum that may be retrieve at a time is
    /// `100`, if a greater number is provided then it is automatically reduced.
    ///
    /// The optional `after` attribute is to retrieve the users after a certain
    /// user. This is useful for pagination.
    ///
    /// **Note**: Requires the [Read Message History] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission
    /// to read messages in the channel.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    pub async fn reaction_users(
        self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        reaction_type: impl Into<ReactionType>,
        limit: Option<u8>,
        after: impl Into<Option<UserId>>,
    ) -> Result<Vec<User>> {
        let limit = limit.map_or(50, |x| if x > 100 { 100 } else { x });

        http.as_ref()
            .get_reaction_users(
                self.get(),
                message_id.into().get(),
                &reaction_type.into(),
                limit,
                after.into().map(UserId::get),
            )
            .await
    }

    /// Sends a message with just the given message content in the channel.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    #[inline]
    pub async fn say(self, http: impl AsRef<Http>, content: impl Into<String>) -> Result<Message> {
        self.send_message(&http, |m| m.content(content)).await
    }

    /// Sends file(s) along with optional message contents. The filename _must_
    /// be specified.
    ///
    /// Message contents may be passed by using the [`CreateMessage::content`]
    /// method.
    ///
    /// The [Attach Files] and [Send Messages] permissions are required.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points, and embeds must be under
    /// 6000 unicode code points.
    ///
    ///
    /// # Examples
    ///
    /// Send files with the paths `/path/to/file.jpg` and `/path/to/file2.jpg`:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() {
    /// # let http = Arc::new(Http::new("token"));
    /// use serenity::model::id::ChannelId;
    ///
    /// let channel_id = ChannelId::new(7);
    ///
    /// let paths = vec!["/path/to/file.jpg", "path/to/file2.jpg"];
    ///
    /// let _ = channel_id.send_files(&http, paths, |m| m.content("a file")).await;
    /// # }
    /// ```
    ///
    /// Send files using [`File`]:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Arc::new(Http::new("token"));
    /// use serenity::model::id::ChannelId;
    /// use tokio::fs::File;
    ///
    /// let channel_id = ChannelId::new(7);
    ///
    /// let f1 = File::open("my_file.jpg").await?;
    /// let f2 = File::open("my_file2.jpg").await?;
    ///
    /// let files = vec![(&f1, "my_file.jpg"), (&f2, "my_file2.jpg")];
    ///
    /// let _ = channel_id.send_files(&http, files, |m| m.content("a file")).await;
    /// #    Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// If the content of the message is over the above limit, then a
    /// [`ModelError::MessageTooLong`] will be returned, containing the number
    /// of unicode code points over the limit.
    ///
    /// Returns an
    /// [`HttpError::UnsuccessfulRequest(ErrorResponse)`][`HttpError::UnsuccessfulRequest`]
    /// if the file(s) are too large to send.
    ///
    /// [`HttpError::UnsuccessfulRequest`]: crate::http::HttpError::UnsuccessfulRequest
    /// [`CreateMessage::content`]: crate::builder::CreateMessage::content
    /// [Attach Files]: Permissions::ATTACH_FILES
    /// [Send Messages]: Permissions::SEND_MESSAGES
    /// [`File`]: tokio::fs::File
    pub async fn send_files<'a, F, T, It>(
        self,
        http: impl AsRef<Http>,
        files: It,
        f: F,
    ) -> Result<Message>
    where
        for<'b> F: FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a>,
        T: Into<AttachmentType<'a>>,
        It: IntoIterator<Item = T>,
    {
        let mut builder = CreateMessage::default();
        http.as_ref().send_files(self.get(), files, f(&mut builder)).await
    }

    /// Sends a message to the channel.
    ///
    /// Refer to the documentation for [`CreateMessage`] for more information
    /// regarding message restrictions and requirements.
    ///
    /// Requires the [Send Messages] permission.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// Returns [`Error::Http`] if the current user lacks permission to
    /// send a message in this channel.
    ///
    /// [Send Messages]: Permissions::SEND_MESSAGES
    pub async fn send_message<'a, F>(self, http: impl AsRef<Http>, f: F) -> Result<Message>
    where
        for<'b> F: FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a>,
    {
        let mut create_message = CreateMessage::default();
        f(&mut create_message);
        self._send_message(http.as_ref(), create_message).await
    }

    async fn _send_message<'a>(self, http: &Http, mut msg: CreateMessage<'a>) -> Result<Message> {
        let files = std::mem::take(&mut msg.files);

        let message = if files.is_empty() {
            http.as_ref().send_message(self.get(), &msg).await?
        } else {
            http.as_ref().send_files(self.get(), files, &msg).await?
        };

        for reaction in msg.reactions {
            self.create_reaction(&http, message.id, reaction).await?;
        }

        Ok(message)
    }

    /// Starts typing in the channel for an indefinite period of time.
    ///
    /// Returns [`Typing`] that is used to trigger the typing. [`Typing::stop`] must be called
    /// on the returned struct to stop typing. Note that on some clients, typing may persist
    /// for a few seconds after [`Typing::stop`] is called.
    /// Typing is also stopped when the struct is dropped.
    ///
    /// If a message is sent while typing is triggered, the user will stop typing for a brief period
    /// of time and then resume again until either [`Typing::stop`] is called or the struct is dropped.
    ///
    /// This should rarely be used for bots, although it is a good indicator that a
    /// long-running command is still being processed.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// # use serenity::{http::{Http, Typing}, Result, model::id::ChannelId};
    /// # use std::sync::Arc;
    /// #
    /// # fn long_process() {}
    /// # fn main() -> Result<()> {
    /// # let http = Arc::new(Http::new("token"));
    /// // Initiate typing (assuming http is `Arc<Http>`)
    /// let typing = ChannelId::new(7).start_typing(&http)?;
    ///
    /// // Run some long-running process
    /// long_process();
    ///
    /// // Stop typing
    /// typing.stop();
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission
    /// to send messages in this channel.
    pub fn start_typing(self, http: &Arc<Http>) -> Result<Typing> {
        http.start_typing(self.get())
    }

    /// Unpins a [`Message`] in the channel given by its Id.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    #[inline]
    pub async fn unpin(
        self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
    ) -> Result<()> {
        http.as_ref().unpin_message(self.get(), message_id.into().get(), None).await
    }

    /// Retrieves the channel's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Webhooks]: Permissions::MANAGE_WEBHOOKS
    #[inline]
    pub async fn webhooks(self, http: impl AsRef<Http>) -> Result<Vec<Webhook>> {
        http.as_ref().get_channel_webhooks(self.get()).await
    }

    /// Creates a webhook
    ///
    /// # Errors
    ///
    /// Returns a [`Error::Http`] if the current user lacks permission.
    /// Returns a [`ModelError::NameTooShort`] if the name of the webhook is
    /// under the limit of 2 characters.
    /// Returns a [`ModelError::NameTooLong`] if the name of the webhook is
    /// over the limit of 100 characters.
    /// Returns a [`ModelError::InvalidChannelType`] if the channel type is not text.
    pub async fn create_webhook<F>(&self, http: impl AsRef<Http>, f: F) -> Result<Webhook>
    where
        F: FnOnce(&mut CreateWebhook) -> &mut CreateWebhook,
    {
        let mut builder = CreateWebhook::default();
        f(&mut builder);

        if let Some(name) = &builder.name {
            if name.len() < 2 {
                return Err(Error::Model(ModelError::NameTooShort));
            } else if name.len() > 100 {
                return Err(Error::Model(ModelError::NameTooLong));
            }
        }

        http.as_ref().create_webhook(self.get(), &builder, None).await
    }

    /// Returns a builder which can be awaited to obtain a message or stream of messages in this channel.
    #[cfg(feature = "collector")]
    pub fn reply_collector<'a>(
        &self,
        shard_messenger: &'a ShardMessenger,
    ) -> MessageCollectorBuilder<'a> {
        MessageCollectorBuilder::new(shard_messenger).channel_id(self.0)
    }

    /// Returns a builder which can be awaited to obtain a reaction or stream of reactions sent in this channel.
    #[cfg(feature = "collector")]
    pub fn reaction_collector<'a>(
        &self,
        shard_messenger: &'a ShardMessenger,
    ) -> ReactionCollectorBuilder<'a> {
        ReactionCollectorBuilder::new(shard_messenger).channel_id(self.0)
    }

    /// Gets a stage instance.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the channel is not a stage channel,
    /// or if there is no stage instance currently.
    pub async fn get_stage_instance(&self, http: impl AsRef<Http>) -> Result<StageInstance> {
        http.as_ref().get_stage_instance(self.get()).await
    }

    /// Creates a stage instance.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the channel is not a stage channel,
    /// or if there is already a stage instance currently.
    pub async fn create_stage_instance<F>(
        &self,
        http: impl AsRef<Http>,
        f: F,
    ) -> Result<StageInstance>
    where
        F: FnOnce(&mut CreateStageInstance) -> &mut CreateStageInstance,
    {
        let mut instance = CreateStageInstance::default();
        f(&mut instance);

        http.as_ref().create_stage_instance(&instance).await
    }

    /// Edits a stage instance.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the channel is not a stage channel,
    /// or if there is not stage instance currently.
    pub async fn edit_stage_instance<F>(
        &self,
        http: impl AsRef<Http>,
        f: F,
    ) -> Result<StageInstance>
    where
        F: FnOnce(&mut EditStageInstance) -> &mut EditStageInstance,
    {
        let mut instance = EditStageInstance::default();
        f(&mut instance);

        http.as_ref().edit_stage_instance(self.get(), &instance).await
    }

    /// Edits a thread.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    pub async fn edit_thread<F>(&self, http: impl AsRef<Http>, f: F) -> Result<GuildChannel>
    where
        F: FnOnce(&mut EditThread) -> &mut EditThread,
    {
        let mut instance = EditThread::default();
        f(&mut instance);

        http.as_ref().edit_thread(self.get(), &instance).await
    }

    /// Deletes a stage instance.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the channel is not a stage channel,
    /// or if there is no stage instance currently.
    pub async fn delete_stage_instance(&self, http: impl AsRef<Http>) -> Result<()> {
        http.as_ref().delete_stage_instance(self.get()).await
    }

    /// Creates a public thread that is connected to a message.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    pub async fn create_public_thread<F>(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        f: F,
    ) -> Result<GuildChannel>
    where
        F: FnOnce(&mut CreateThread) -> &mut CreateThread,
    {
        let mut instance = CreateThread::default();
        f(&mut instance);

        http.as_ref().create_public_thread(self.get(), message_id.into().get(), &instance).await
    }

    /// Creates a private thread.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    pub async fn create_private_thread<F>(
        &self,
        http: impl AsRef<Http>,
        f: F,
    ) -> Result<GuildChannel>
    where
        F: FnOnce(&mut CreateThread) -> &mut CreateThread,
    {
        let mut instance = CreateThread::default();
        instance.kind(ChannelType::PrivateThread);
        f(&mut instance);

        http.as_ref().create_private_thread(self.get(), &instance).await
    }

    /// Gets the thread members, if this channel is a thread.
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the channel is not a thread channel
    pub async fn get_thread_members(&self, http: impl AsRef<Http>) -> Result<Vec<ThreadMember>> {
        http.as_ref().get_channel_thread_members(self.get()).await
    }

    /// Joins the thread, if this channel is a thread.
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the channel is not a thread channel
    pub async fn join_thread(&self, http: impl AsRef<Http>) -> Result<()> {
        http.as_ref().join_thread_channel(self.get()).await
    }

    /// Leaves the thread, if this channel is a thread.
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the channel is not a thread channel
    pub async fn leave_thread(&self, http: impl AsRef<Http>) -> Result<()> {
        http.as_ref().leave_thread_channel(self.get()).await
    }

    /// Adds a thread member, if this channel is a thread.
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the channel is not a thread channel
    pub async fn add_thread_member(&self, http: impl AsRef<Http>, user_id: UserId) -> Result<()> {
        http.as_ref().add_thread_channel_member(self.get(), user_id.into()).await
    }

    /// Removes a thread member, if this channel is a thread.
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the channel is not a thread channel
    pub async fn remove_thread_member(
        &self,
        http: impl AsRef<Http>,
        user_id: UserId,
    ) -> Result<()> {
        http.as_ref().remove_thread_channel_member(self.get(), user_id.into()).await
    }

    /// Gets private archived threads of a channel.
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the bot doesn't have the
    /// permission to get it.
    pub async fn get_archived_private_threads(
        &self,
        http: impl AsRef<Http>,
        before: Option<u64>,
        limit: Option<u64>,
    ) -> Result<ThreadsData> {
        http.as_ref().get_channel_archived_private_threads(self.get(), before, limit).await
    }

    /// Gets public archived threads of a channel.
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the bot doesn't have the
    /// permission to get it.
    pub async fn get_archived_public_threads(
        &self,
        http: impl AsRef<Http>,
        before: Option<u64>,
        limit: Option<u64>,
    ) -> Result<ThreadsData> {
        http.as_ref().get_channel_archived_public_threads(self.get(), before, limit).await
    }

    /// Gets private archived threads joined by the current user of a channel.
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the bot doesn't have the
    /// permission to get it.
    pub async fn get_joined_archived_private_threads(
        &self,
        http: impl AsRef<Http>,
        before: Option<u64>,
        limit: Option<u64>,
    ) -> Result<ThreadsData> {
        http.as_ref().get_channel_joined_archived_private_threads(self.get(), before, limit).await
    }
}

#[cfg(feature = "model")]
impl From<Channel> for ChannelId {
    /// Gets the Id of a [`Channel`].
    fn from(channel: Channel) -> ChannelId {
        channel.id()
    }
}

#[cfg(feature = "model")]
impl<'a> From<&'a Channel> for ChannelId {
    /// Gets the Id of a [`Channel`].
    fn from(channel: &Channel) -> ChannelId {
        channel.id()
    }
}

impl From<PrivateChannel> for ChannelId {
    /// Gets the Id of a private channel.
    fn from(private_channel: PrivateChannel) -> ChannelId {
        private_channel.id
    }
}

impl<'a> From<&'a PrivateChannel> for ChannelId {
    /// Gets the Id of a private channel.
    fn from(private_channel: &PrivateChannel) -> ChannelId {
        private_channel.id
    }
}

impl From<GuildChannel> for ChannelId {
    /// Gets the Id of a guild channel.
    fn from(public_channel: GuildChannel) -> ChannelId {
        public_channel.id
    }
}

impl<'a> From<&'a GuildChannel> for ChannelId {
    /// Gets the Id of a guild channel.
    fn from(public_channel: &GuildChannel) -> ChannelId {
        public_channel.id
    }
}

/// A helper class returned by [`ChannelId::messages_iter`]
#[derive(Clone, Debug)]
#[cfg(feature = "model")]
pub struct MessagesIter<H: AsRef<Http>> {
    http: H,
    channel_id: ChannelId,
    buffer: Vec<Message>,
    before: Option<MessageId>,
    tried_fetch: bool,
}

#[cfg(feature = "model")]
impl<H: AsRef<Http>> MessagesIter<H> {
    fn new(http: H, channel_id: ChannelId) -> MessagesIter<H> {
        MessagesIter {
            http,
            channel_id,
            buffer: Vec::new(),
            before: None,
            tried_fetch: false,
        }
    }

    /// Fills the `self.buffer` cache with [`Message`]s.
    ///
    /// This drops any messages that were currently in the buffer. Ideally, it
    /// should only be called when `self.buffer` is empty. Additionally, this updates
    /// `self.before` so that the next call does not return duplicate items.
    ///
    /// If there are no more messages to be fetched, then this sets `self.before`
    /// as [`None`], indicating that no more calls ought to be made.
    ///
    /// If this method is called with `self.before` as None, the last 100
    /// (or lower) messages sent in the channel are added in the buffer.
    ///
    /// The messages are sorted such that the newest message is the first
    /// element of the buffer and the newest message is the last.
    ///
    /// [`Message`]: crate::model::channel::Message
    async fn refresh(&mut self) -> Result<()> {
        // Number of messages to fetch.
        let grab_size = 100;

        // If `self.before` is not set yet, we can use `.messages` to fetch
        // the last message after very first fetch from last.
        self.buffer = self
            .channel_id
            .messages(&self.http, |b| {
                if let Some(before) = self.before {
                    b.before(before);
                }

                b.limit(grab_size)
            })
            .await?;

        self.buffer.reverse();

        self.before = self.buffer.first().map(|m| m.id);

        self.tried_fetch = true;

        Ok(())
    }

    /// Streams over all the messages in a channel.
    ///
    /// This is accomplished and equivalent to repeated calls to [`ChannelId::messages`].
    /// A buffer of at most 100 messages is used to reduce the number of calls.
    /// necessary.
    ///
    /// The stream returns the newest message first, followed by older messages.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use serenity::model::id::ChannelId;
    /// # use serenity::http::Http;
    /// #
    /// # async fn run() {
    /// # let channel_id = ChannelId::new(1);
    /// # let ctx = Http::new("token");
    /// use serenity::futures::StreamExt;
    /// use serenity::model::channel::MessagesIter;
    ///
    /// let mut messages = MessagesIter::<Http>::stream(&ctx, channel_id).boxed();
    /// while let Some(message_result) = messages.next().await {
    ///     match message_result {
    ///         Ok(message) => println!("{} said \"{}\"", message.author.name, message.content,),
    ///         Err(error) => eprintln!("Uh oh! Error: {}", error),
    ///     }
    /// }
    /// # }
    /// ```
    pub fn stream(
        http: impl AsRef<Http>,
        channel_id: ChannelId,
    ) -> impl Stream<Item = Result<Message>> {
        let init_state = MessagesIter::new(http, channel_id);

        futures::stream::unfold(init_state, |mut state| async {
            if state.buffer.is_empty() && state.before.is_some() || !state.tried_fetch {
                if let Err(error) = state.refresh().await {
                    return Some((Err(error), state));
                }
            }

            // the resultant stream goes from newest to oldest.
            state.buffer.pop().map(|entry| (Ok(entry), state))
        })
    }
}
