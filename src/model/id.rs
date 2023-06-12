//! A collection of newtypes defining type-strong IDs.

use std::fmt;
use std::num::NonZeroU64;

use super::Timestamp;

struct SnowflakeVisitor;

impl<'de> serde::de::Visitor<'de> for SnowflakeVisitor {
    type Value = NonZeroU64;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a non-zero string or integer snowflake")
    }

    // Called by formats like TOML.
    fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Self::Value, E> {
        self.visit_u64(u64::try_from(value).map_err(E::custom)?)
    }

    fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Self::Value, E> {
        NonZeroU64::new(value).ok_or_else(|| E::custom("invalid value, expected non-zero"))
    }

    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
        value.parse().map_err(E::custom)
    }
}

macro_rules! id_u64 {
    ($($name:ident;)*) => {
        $(
            impl $name {
                /// Creates a new Id from a u64
                ///
                /// # Panics
                /// Panics if the id is zero.
                #[inline]
                #[must_use]
                #[track_caller]
                pub const fn new(id: u64) -> Self {
                    Self(id.to_be_bytes())
                }

                /// Retrieves the inner ID as u64
                #[inline]
                #[must_use]
                pub const fn get(self) -> u64 {
                    u64::from_be_bytes(self.0)
                }

                /// Retrieves the time that the Id was created at.
                #[must_use]
                pub fn created_at(&self) -> Timestamp {
                    Timestamp::from_discord_id(self.get())
                }
            }

            impl Default for $name {
                fn default() -> Self {
                    Self::new(1)
                }
            }

            // This is a hack so functions can accept iterators that either:
            // 1. return the id itself (e.g: `MessageId`)
            // 2. return a reference to it (`&MessageId`).
            impl AsRef<$name> for $name {
                fn as_ref(&self) -> &Self {
                    self
                }
            }

            impl<'a> From<&'a $name> for $name {
                fn from(id: &'a $name) -> $name {
                    id.clone()
                }
            }

            impl From<u64> for $name {
                fn from(id: u64) -> $name {
                    Self::new(id)
                }
            }

            impl PartialEq<u64> for $name {
                fn eq(&self, u: &u64) -> bool {
                    self.get() == *u
                }
            }

            impl fmt::Display for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    fmt::Display::fmt(&self.get(), f)
                }
            }

            impl From<$name> for u64 {
                fn from(id: $name) -> u64 {
                    id.get()
                }
            }

            impl From<$name> for i64 {
                fn from(id: $name) -> i64 {
                    id.get() as i64
                }
            }

            impl serde::Serialize for $name {
                fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                    serializer.collect_str(&self.get())
                }
            }

            impl<'de> serde::Deserialize<'de> for $name {
                fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                    deserializer.deserialize_any(SnowflakeVisitor).map(NonZeroU64::get).map(Self::new)
                }
            }
        )*
    }
}

type IdInner = [u8; 8];

/// An identifier for an Application.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ApplicationId(pub(crate) IdInner);

/// An identifier for a Channel
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ChannelId(pub(crate) IdInner);

/// An identifier for an Emoji
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EmojiId(pub(crate) IdInner);

/// An identifier for an unspecific entity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GenericId(pub(crate) IdInner);

/// An identifier for a Guild
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GuildId(pub(crate) IdInner);

/// An identifier for an Integration
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IntegrationId(pub(crate) IdInner);

/// An identifier for a Message
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MessageId(pub(crate) IdInner);

/// An identifier for a Role
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RoleId(pub(crate) IdInner);

/// An identifier for an auto moderation rule
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct RuleId(pub(crate) IdInner);

/// An identifier for a Scheduled Event
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScheduledEventId(pub(crate) IdInner);

/// An identifier for a User
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UserId(pub(crate) IdInner);

/// An identifier for a [`Webhook`][super::webhook::Webhook]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WebhookId(pub(crate) IdInner);

/// An identifier for an audit log entry.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AuditLogEntryId(pub(crate) IdInner);

/// An identifier for an attachment.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AttachmentId(pub(crate) IdInner);

/// An identifier for a sticker.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StickerId(pub(crate) IdInner);

/// An identifier for a sticker pack.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StickerPackId(pub(crate) IdInner);

/// An identifier for a sticker pack banner.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StickerPackBannerId(pub(crate) IdInner);

/// An identifier for a SKU.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SkuId(pub(crate) IdInner);

/// An identifier for an interaction.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InteractionId(pub(crate) IdInner);

/// An identifier for a slash command.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CommandId(pub(crate) IdInner);

/// An identifier for a slash command permission Id. Can contain
/// a [`RoleId`] or [`UserId`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CommandPermissionId(pub(crate) IdInner);

/// An identifier for a slash command version Id.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CommandVersionId(pub(crate) IdInner);

/// An identifier for a slash command target Id. Can contain
/// a [`UserId`] or [`MessageId`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TargetId(pub(crate) IdInner);

/// An identifier for a stage channel instance.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StageInstanceId(pub(crate) IdInner);

/// An identifier for a forum tag.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct ForumTagId(pub(crate) IdInner);

id_u64! {
    AttachmentId;
    ApplicationId;
    ChannelId;
    EmojiId;
    GenericId;
    GuildId;
    IntegrationId;
    MessageId;
    RoleId;
    ScheduledEventId;
    StickerId;
    StickerPackId;
    StickerPackBannerId;
    SkuId;
    UserId;
    WebhookId;
    AuditLogEntryId;
    InteractionId;
    CommandId;
    CommandPermissionId;
    CommandVersionId;
    TargetId;
    StageInstanceId;
    RuleId;
    ForumTagId;
}

#[cfg(test)]
mod tests {
    use super::GuildId;

    #[test]
    fn test_created_at() {
        // The id is from discord's snowflake docs
        let id = GuildId::new(175928847299117063);
        assert_eq!(id.created_at().unix_timestamp(), 1462015105);
        assert_eq!(id.created_at().to_string(), "2016-04-30T11:18:25.796Z");
    }

    #[test]
    fn test_id_serde() {
        use crate::json::{assert_json, json};

        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct Opt {
            id: Option<GuildId>,
        }

        let id = GuildId::new(17_5928_8472_9911_7063);
        assert_json(&id, json!("175928847299117063"));

        let s = Opt {
            id: Some(GuildId::new(17_5928_8472_9911_7063)),
        };
        assert_json(&s, json!({"id": "175928847299117063"}));
    }
}
