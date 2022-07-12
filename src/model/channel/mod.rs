//! Models relating to channels and types within channels.

mod attachment;
mod attachment_type;
mod channel_category;
mod channel_id;
mod embed;
mod guild_channel;
mod message;
mod partial_channel;
mod private_channel;
mod reaction;

#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use std::error::Error as StdError;
use std::fmt;
use std::num::NonZeroU64;

use serde::de::{Error as DeError, Unexpected};
use serde::ser::{Serialize, Serializer};

pub use self::attachment::*;
pub use self::attachment_type::*;
pub use self::channel_category::*;
pub use self::channel_id::*;
pub use self::embed::*;
pub use self::guild_channel::*;
pub use self::message::*;
pub use self::partial_channel::*;
pub use self::private_channel::*;
pub use self::reaction::*;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::Cache;
#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use crate::cache::FromStrAndCache;
#[cfg(feature = "model")]
use crate::http::CacheHttp;
use crate::json::prelude::*;
use crate::model::prelude::*;
use crate::model::utils::is_false;
use crate::model::Timestamp;
#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use crate::utils::parse_channel;

/// A container for any channel.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Channel {
    /// A [text], [voice], [stage] or [directory] channel within a [`Guild`].
    ///
    /// [text]: ChannelType::Text
    /// [voice]: ChannelType::Voice
    /// [stage]: ChannelType::Stage
    /// [directory]: ChannelType::Directory
    Guild(GuildChannel),
    /// A private channel to another [`User`]. No other users may access the
    /// channel. For multi-user "private channels", use a group.
    Private(PrivateChannel),
    /// A category of [`GuildChannel`]s
    Category(ChannelCategory),
}

impl Channel {
    /// Converts from [`Channel`] to `Option<GuildChannel>`.
    ///
    /// Converts `self` into an `Option<GuildChannel>`, consuming
    /// `self`, and discarding a [`PrivateChannel`], or
    /// [`ChannelCategory`], if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// # use serenity::model::channel::Channel;
    /// # fn run(channel: Channel) {
    /// #
    /// match channel.guild() {
    ///     Some(guild_channel) => {
    ///         println!("It's a guild channel named {}!", guild_channel.name);
    ///     },
    ///     None => {
    ///         println!("It's not in a guild!");
    ///     },
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn guild(self) -> Option<GuildChannel> {
        match self {
            Self::Guild(lock) => Some(lock),
            _ => None,
        }
    }

    /// Converts from [`Channel`] to `Option<PrivateChannel>`.
    ///
    /// Converts `self` into an `Option<PrivateChannel>`, consuming
    /// `self`, and discarding a [`GuildChannel`], or [`ChannelCategory`],
    /// if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// # use serenity::model::channel::Channel;
    /// # fn run(channel: Channel) {
    /// #
    /// match channel.private() {
    ///     Some(private) => {
    ///         println!("It's a private channel with {}!", &private.recipient);
    ///     },
    ///     None => {
    ///         println!("It's not a private channel!");
    ///     },
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn private(self) -> Option<PrivateChannel> {
        match self {
            Self::Private(lock) => Some(lock),
            _ => None,
        }
    }

    /// Converts from [`Channel`] to `Option<ChannelCategory>`.
    ///
    /// Converts `self` into an `Option<ChannelCategory>`,
    /// consuming `self`, and discarding a [`GuildChannel`], or
    /// [`PrivateChannel`], if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// # use serenity::model::channel::Channel;
    /// # fn run(channel: Channel) {
    /// #
    /// match channel.category() {
    ///     Some(category) => {
    ///         println!("It's a category named {}!", category.name);
    ///     },
    ///     None => {
    ///         println!("It's not a category!");
    ///     },
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn category(self) -> Option<ChannelCategory> {
        match self {
            Self::Category(lock) => Some(lock),
            _ => None,
        }
    }

    /// Deletes the inner channel.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns [`ModelError::InvalidPermissions`],
    /// if the current user lacks permission.
    ///
    /// Otherwise will return [`Error::Http`] if the current user does not
    /// have permission.
    #[cfg(feature = "http")]
    pub async fn delete(&self, cache_http: impl CacheHttp) -> Result<()> {
        match self {
            Self::Guild(public_channel) => {
                public_channel.delete(cache_http).await?;
            },
            Self::Private(private_channel) => {
                private_channel.delete(cache_http.http()).await?;
            },
            Self::Category(category) => {
                category.delete(cache_http).await?;
            },
        }

        Ok(())
    }

    /// Determines if the channel is NSFW.
    #[inline]
    #[must_use]
    #[cfg(feature = "model")]
    pub fn is_nsfw(&self) -> bool {
        match self {
            Self::Guild(channel) => channel.is_nsfw(),
            Self::Category(category) => category.is_nsfw(),
            Self::Private(_) => false,
        }
    }

    /// Retrieves the Id of the inner [`GuildChannel`], or
    /// [`PrivateChannel`].
    #[inline]
    #[must_use]
    pub const fn id(&self) -> ChannelId {
        match self {
            Self::Guild(ch) => ch.id,
            Self::Private(ch) => ch.id,
            Self::Category(ch) => ch.id,
        }
    }

    /// Retrieves the position of the inner [`GuildChannel`] or
    /// [`ChannelCategory`].
    ///
    /// If other channel types are used it will return None.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> Option<i64> {
        match self {
            Self::Guild(channel) => Some(channel.position),
            Self::Category(category) => Some(category.position),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for Channel {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        let map = value.as_object().ok_or_else(|| DeError::custom("expected a JsonMap"))?;

        let kind = {
            let kind = map.get("type").ok_or_else(|| DeError::missing_field("type"))?;
            kind.as_u64().ok_or_else(|| {
                DeError::invalid_type(
                    Unexpected::Other("non-positive integer"),
                    &"a positive integer",
                )
            })?
        };

        match kind {
            0 | 2 | 5 | 10 | 11 | 12 | 13 | 14 | 15 => from_value(value).map(Channel::Guild),
            1 => from_value(value).map(Channel::Private),
            4 => from_value(value).map(Channel::Category),
            _ => return Err(DeError::custom("Unknown channel type")),
        }
        .map_err(DeError::custom)
    }
}

impl Serialize for Channel {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Category(c) => ChannelCategory::serialize(c, serializer),
            Self::Guild(c) => GuildChannel::serialize(c, serializer),
            Self::Private(c) => PrivateChannel::serialize(c, serializer),
        }
    }
}

impl fmt::Display for Channel {
    /// Formats the channel into a "mentioned" string.
    ///
    /// This will return a different format for each type of channel:
    ///
    /// - [`PrivateChannel`]s: the recipient's name;
    /// - [`GuildChannel`]s: a string mentioning the channel that users who can
    /// see the channel can click on.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Guild(ch) => fmt::Display::fmt(&ch.id.mention(), f),
            Self::Private(ch) => fmt::Display::fmt(&ch.recipient.name, f),
            Self::Category(ch) => fmt::Display::fmt(&ch.name, f),
        }
    }
}

enum_number! {
    /// A representation of a type of channel.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-types).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum ChannelType {
        /// An indicator that the channel is a text [`GuildChannel`].
        Text = 0,
        /// An indicator that the channel is a [`PrivateChannel`].
        Private = 1,
        /// An indicator that the channel is a voice [`GuildChannel`].
        Voice = 2,
        /// An indicator that the channel is the channel of a [`ChannelCategory`].
        Category = 4,
        /// An indicator that the channel is a `NewsChannel`.
        ///
        /// Note: `NewsChannel` is serialized into a [`GuildChannel`]
        News = 5,
        /// An indicator that the channel is a news thread [`GuildChannel`].
        NewsThread = 10,
        /// An indicator that the channel is a public thread [`GuildChannel`].
        PublicThread = 11,
        /// An indicator that the channel is a private thread [`GuildChannel`].
        PrivateThread = 12,
        /// An indicator that the channel is a stage [`GuildChannel`].
        Stage = 13,
        /// An indicator that the channel is a directory [`GuildChannel`] in a [hub].
        ///
        /// [hub]: https://support.discord.com/hc/en-us/articles/4406046651927-Discord-Student-Hubs-FAQ
        Directory = 14,
        /// An indicator that the channel is a forum [`GuildChannel`].
        #[cfg(feature = "unstable_discord_api")]
        Forum = 15,
        _ => Unknown(u8),
    }
}

impl ChannelType {
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &str {
        match *self {
            Self::Private => "private",
            Self::Text => "text",
            Self::Voice => "voice",
            Self::Category => "category",
            Self::News => "news",
            Self::NewsThread => "news_thread",
            Self::PublicThread => "public_thread",
            Self::PrivateThread => "private_thread",
            Self::Stage => "stage",
            Self::Directory => "directory",
            #[cfg(feature = "unstable_discord_api")]
            Self::Forum => "forum",
            Self::Unknown(_) => "unknown",
        }
    }
}

/// [Discord docs](https://discord.com/developers/docs/resources/channel#overwrite-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct PermissionOverwriteData {
    allow: Permissions,
    deny: Permissions,
    #[serde(with = "snowflake")]
    id: NonZeroU64,
    #[serde(rename = "type")]
    kind: u8,
}

pub(crate) struct InvalidPermissionOverwriteType(u8);

impl std::fmt::Display for InvalidPermissionOverwriteType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid Permission Overwrite Type: {}", self.0)
    }
}

impl std::convert::TryFrom<PermissionOverwriteData> for PermissionOverwrite {
    type Error = InvalidPermissionOverwriteType;

    fn try_from(data: PermissionOverwriteData) -> StdResult<Self, Self::Error> {
        let kind = match data.kind {
            0 => PermissionOverwriteType::Role(RoleId(data.id)),
            1 => PermissionOverwriteType::Member(UserId(data.id)),
            raw => return Err(InvalidPermissionOverwriteType(raw)),
        };

        Ok(PermissionOverwrite {
            allow: data.allow,
            deny: data.deny,
            kind,
        })
    }
}

impl From<PermissionOverwrite> for PermissionOverwriteData {
    fn from(data: PermissionOverwrite) -> Self {
        let (kind, id) = match data.kind {
            PermissionOverwriteType::Role(id) => (0, id.0),
            PermissionOverwriteType::Member(id) => (1, id.0),
        };

        PermissionOverwriteData {
            allow: data.allow,
            deny: data.deny,
            kind,
            id,
        }
    }
}

/// A channel-specific permission overwrite for a member or role.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#overwrite-object).
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(try_from = "PermissionOverwriteData", into = "PermissionOverwriteData")]
pub struct PermissionOverwrite {
    pub allow: Permissions,
    pub deny: Permissions,
    pub kind: PermissionOverwriteType,
}

/// The type of edit being made to a Channel's permissions.
///
/// This is for use with methods such as [`GuildChannel::create_permission`].
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#overwrite-object-overwrite-structure) (field `type`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PermissionOverwriteType {
    /// A member which is having its permission overwrites edited.
    Member(UserId),
    /// A role which is having its permission overwrites edited.
    Role(RoleId),
}

enum_number! {
    /// The video quality mode for a voice channel.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/channel#channel-object-video-quality-modes).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum VideoQualityMode {
        /// An indicator that the video quality is chosen by Discord for optimal
        /// performance.
        Auto = 1,
        /// An indicator that the video quality is 720p.
        Full = 2,
        _ => Unknown(u8),
    }
}

/// [Discord docs](https://discord.com/developers/docs/resources/stage-instance#stage-instance-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct StageInstance {
    /// The Id of the stage instance.
    pub id: StageInstanceId,
    /// The guild Id of the associated stage channel.
    pub guild_id: GuildId,
    /// The Id of the associated stage channel.
    pub channel_id: ChannelId,
    /// The topic of the stage instance.
    pub topic: String,
}

/// A thread data.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#thread-metadata-object).
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ThreadMetadata {
    /// Whether the thread is archived.
    pub archived: bool,
    /// Duration in minutes to automatically archive the thread after recent activity.
    ///
    /// **Note**: It can currently only be set to 60, 1440, 4320, 10080.
    pub auto_archive_duration: Option<u64>,
    /// Timestamp when the thread's archive status was last changed, used for calculating recent activity.
    pub archive_timestamp: Option<Timestamp>,
    /// When a thread is locked, only users with `MANAGE_THREADS` permission can unarchive it.
    #[serde(default)]
    pub locked: bool,
    /// Timestamp when the thread was created.
    ///
    /// **Note**: only populated for threads created after 2022-01-09
    pub create_timestamp: Option<Timestamp>,
    /// Whether non-moderators can add other non-moderators to a thread.
    ///
    /// **Note**: Only available on private threads.
    #[serde(default, skip_serializing_if = "is_false")]
    pub invitable: bool,
}

/// A response to getting several threads channels.
///
/// Discord docs: scattered, but e.g. [here](https://discord.com/developers/docs/resources/channel#list-public-archived-threads-response-body).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ThreadsData {
    /// The threads channels.
    pub threads: Vec<GuildChannel>,
    /// A thread member for each returned thread the current user has joined.
    pub members: Vec<ThreadMember>,
    /// Whether there are potentially additional threads that could be returned on a subsequent call.
    #[serde(default)]
    pub has_more: bool,
}

#[cfg(test)]
mod test {
    #[cfg(all(feature = "model", feature = "utils"))]
    mod model_utils {
        use crate::model::prelude::*;

        fn guild_channel() -> GuildChannel {
            GuildChannel {
                id: ChannelId::new(1),
                bitrate: None,
                parent_id: None,
                guild_id: GuildId::new(2),
                kind: ChannelType::Text,
                last_message_id: None,
                last_pin_timestamp: None,
                name: "nsfw-stuff".to_string(),
                permission_overwrites: vec![],
                position: 0,
                topic: None,
                user_limit: None,
                nsfw: false,
                rate_limit_per_user: Some(0),
                rtc_region: None,
                video_quality_mode: None,
                message_count: None,
                member_count: None,
                thread_metadata: None,
                member: None,
                default_auto_archive_duration: None,
            }
        }

        fn private_channel() -> PrivateChannel {
            PrivateChannel {
                id: ChannelId::new(1),
                last_message_id: None,
                last_pin_timestamp: None,
                kind: ChannelType::Private,
                recipient: User {
                    id: UserId::new(2),
                    avatar: None,
                    bot: false,
                    discriminator: 1,
                    name: "ab".to_string(),
                    public_flags: None,
                    banner: None,
                    accent_colour: None,
                },
            }
        }

        #[test]
        fn nsfw_checks() {
            let mut channel = guild_channel();
            assert!(!channel.is_nsfw());
            channel.kind = ChannelType::Voice;
            assert!(!channel.is_nsfw());

            channel.kind = ChannelType::Text;
            channel.name = "nsfw-".to_string();
            assert!(!channel.is_nsfw());

            channel.name = "nsfw".to_string();
            assert!(!channel.is_nsfw());
            channel.kind = ChannelType::Voice;
            assert!(!channel.is_nsfw());
            channel.kind = ChannelType::Text;

            channel.name = "nsf".to_string();
            channel.nsfw = true;
            assert!(channel.is_nsfw());
            channel.nsfw = false;
            assert!(!channel.is_nsfw());

            let channel = Channel::Guild(channel);
            assert!(!channel.is_nsfw());

            let private_channel = private_channel();
            assert!(!private_channel.is_nsfw());
        }
    }
}

#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
#[derive(Debug)]
pub enum ChannelParseError {
    NotPresentInCache,
    InvalidChannel,
}

#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
impl fmt::Display for ChannelParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotPresentInCache => f.write_str("not present in cache"),
            Self::InvalidChannel => f.write_str("invalid channel"),
        }
    }
}

#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
impl StdError for ChannelParseError {}

#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
impl FromStrAndCache for Channel {
    type Err = ChannelParseError;

    fn from_str<C>(cache: C, s: &str) -> StdResult<Self, Self::Err>
    where
        C: AsRef<Cache> + Send + Sync,
    {
        match parse_channel(s) {
            Some(x) => match x.to_channel_cached(&cache) {
                Some(channel) => Ok(channel),
                _ => Err(ChannelParseError::NotPresentInCache),
            },
            _ => Err(ChannelParseError::InvalidChannel),
        }
    }
}
