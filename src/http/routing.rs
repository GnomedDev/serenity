use std::borrow::Cow;

use crate::model::id::*;

/// Used to group requests together for ratelimiting.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RatelimitingBucket(Option<(std::mem::Discriminant<Route<'static>>, Option<GenericId>)>);

impl RatelimitingBucket {
    #[must_use]
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

enum RatelimitingKind {
    /// Requests with the same path and major parameter (usually an Id) should be grouped together
    /// for ratelimiting.
    PathAndId(GenericId),
    /// Requests with the same path should be ratelimited together.
    Path,
}

/// A macro for defining routes as well as the type of ratelimiting they perform. Takes as input a
/// list of route definitions, and generates a definition for the `Route` enum and implements
/// methods on it.
macro_rules! routes {
    ($lt:lifetime, {
        $(
            $name:ident $({ $($field_name:ident: $field_type:ty),* })?,
            $path:expr,
            $ratelimiting_kind:expr;
        )+
    }) => {
        #[derive(Clone, Copy, Debug)]
        pub enum Route<$lt> {
            $(
                $name $({ $($field_name: $field_type),* })?,
            )+
        }

        impl<$lt> Route<$lt> {
            #[must_use]
            pub fn path(&self) -> Cow<'static, str> {
                match self {
                    $(
                        Self::$name $({ $($field_name),* })? => $path.into(),
                    )+
                }
            }

            #[must_use]
            pub fn ratelimiting_bucket(&self) -> RatelimitingBucket {
                #[allow(unused_variables)]
                let ratelimiting_kind = match self {
                    $(
                        Self::$name $({ $($field_name),* })? => $ratelimiting_kind,
                    )+
                };

                // To avoid adding a lifetime on RatelimitingBucket and causing lifetime infection,
                // we transmute the Discriminant<Route<'a>> to Discriminant<Route<'static>>.
                // SAFETY: std::mem::discriminant erases lifetimes.
                let discriminant = unsafe { std::mem::transmute(std::mem::discriminant(self)) };

                RatelimitingBucket(ratelimiting_kind.map(|r| {
                    let id = match r {
                        RatelimitingKind::PathAndId(id) => Some(id),
                        RatelimitingKind::Path => None,
                    };
                    (discriminant, id)
                }))
            }

        }
    };
}

// This macro takes as input a list of route definitions, represented in the following way:
// 1. The first line defines an enum variant representing an endpoint.
// 2. The second line provides the url for that endpoint.
// 3. The third line indicates what type of ratelimiting the endpoint employs.
routes! ('a, {
    Channel { channel_id: ChannelId },
    api!("/channels/{}", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelInvites { channel_id: ChannelId },
    api!("/channels/{}/invites", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelMessage { channel_id: ChannelId, message_id: MessageId },
    api!("/channels/{}/messages/{}", channel_id, message_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelMessageCrosspost { channel_id: ChannelId, message_id: MessageId },
    api!("/channels/{}/messages/{}/crosspost", channel_id, message_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelMessageReaction { channel_id: ChannelId, message_id: MessageId, user_id: UserId, reaction: &'a str },
    api!("/channels/{}/messages/{}/reactions/{}/{}", channel_id, message_id, reaction, user_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelMessageReactionMe { channel_id: ChannelId, message_id: MessageId, reaction: &'a str },
    api!("/channels/{}/messages/{}/reactions/{}/@me", channel_id, message_id, reaction),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelMessageReactionEmoji { channel_id: ChannelId, message_id: MessageId, reaction: &'a str },
    api!("/channels/{}/messages/{}/reactions/{}", channel_id, message_id, reaction),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelMessageReactions { channel_id: ChannelId, message_id: MessageId },
    api!("/channels/{}/messages/{}/reactions", channel_id, message_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelMessages { channel_id: ChannelId },
    api!("/channels/{}/messages", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelMessagesBulkDelete { channel_id: ChannelId },
    api!("/channels/{}/messages/bulk-delete", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelFollowNews { channel_id: ChannelId },
    api!("/channels/{}/followers", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelPermission { channel_id: ChannelId, target_id: TargetId },
    api!("/channels/{}/permissions/{}", channel_id, target_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelPin { channel_id: ChannelId, message_id: MessageId },
    api!("/channels/{}/pins/{}", channel_id, message_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelPins { channel_id: ChannelId },
    api!("/channels/{}/pins", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelTyping { channel_id: ChannelId },
    api!("/channels/{}/typing", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelWebhooks { channel_id: ChannelId },
    api!("/channels/{}/webhooks", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelPublicThreads { channel_id: ChannelId, message_id: MessageId },
    api!("/channels/{}/messages/{}/threads", channel_id, message_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelPrivateThreads { channel_id: ChannelId },
    api!("/channels/{}/threads", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelForumPosts { channel_id: ChannelId },
    api!("/channels/{}/threads", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelThreadMember { channel_id: ChannelId, user_id: UserId },
    api!("/channels/{}/thread-members/{}", channel_id, user_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelThreadMemberMe { channel_id: ChannelId },
    api!("/channels/{}/thread-members/@me", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelThreadMembers { channel_id: ChannelId },
    api!("/channels/{}/thread-members", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelArchivedPublicThreads { channel_id: ChannelId },
    api!("/channels/{}/threads/archived/public", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelArchivedPrivateThreads { channel_id: ChannelId },
    api!("/channels/{}/threads/archived/private", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    ChannelJoinedPrivateThreads { channel_id: ChannelId },
    api!("/channels/{}/users/@me/threads/archived/private", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id.get().into()));

    Gateway,
    api!("/gateway"),
    Some(RatelimitingKind::Path);

    GatewayBot,
    api!("/gateway/bot"),
    Some(RatelimitingKind::Path);

    Guild { guild_id: GuildId },
    api!("/guilds/{}", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildAuditLogs { guild_id: GuildId },
    api!("/guilds/{}/audit-logs", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildAutomodRule { guild_id: GuildId, rule_id: RuleId },
    api!("/guilds/{}/auto-moderation/rules/{}", guild_id, rule_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildAutomodRules { guild_id: GuildId },
    api!("/guilds/{}/auto-moderation/rules", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildBan { guild_id: GuildId, user_id: UserId },
    api!("/guilds/{}/bans/{}", guild_id, user_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildBans { guild_id: GuildId },
    api!("/guilds/{}/bans", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildChannels { guild_id: GuildId },
    api!("/guilds/{}/channels", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildWidget { guild_id: GuildId },
    api!("/guilds/{}/widget", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildPreview { guild_id: GuildId },
    api!("/guilds/{}/preview", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildEmojis { guild_id: GuildId },
    api!("/guilds/{}/emojis", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildEmoji { guild_id: GuildId, emoji_id: EmojiId },
    api!("/guilds/{}/emojis/{}", guild_id, emoji_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildIntegration { guild_id: GuildId, integration_id: IntegrationId },
    api!("/guilds/{}/integrations/{}", guild_id, integration_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildIntegrationSync { guild_id: GuildId, integration_id: IntegrationId },
    api!("/guilds/{}/integrations/{}/sync", guild_id, integration_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildIntegrations { guild_id: GuildId },
    api!("/guilds/{}/integrations", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildInvites { guild_id: GuildId },
    api!("/guilds/{}/invites", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildMember { guild_id: GuildId, user_id: UserId },
    api!("/guilds/{}/members/{}", guild_id, user_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildMemberRole { guild_id: GuildId, user_id: UserId, role_id: RoleId },
    api!("/guilds/{}/members/{}/roles/{}", guild_id, user_id, role_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildMembers { guild_id: GuildId },
    api!("/guilds/{}/members", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildMembersSearch { guild_id: GuildId },
    api!("/guilds/{}/members/search", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildMemberMe { guild_id: GuildId },
    api!("/guilds/{}/members/@me", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildMfa { guild_id: GuildId },
    api!("/guilds/{}/mfa", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildPrune { guild_id: GuildId },
    api!("/guilds/{}/prune", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildRegions { guild_id: GuildId },
    api!("/guilds/{}/regions", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildRole { guild_id: GuildId, role_id: RoleId },
    api!("/guilds/{}/roles/{}", guild_id, role_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildRoles { guild_id: GuildId },
    api!("/guilds/{}/roles", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildScheduledEvent { guild_id: GuildId, event_id: ScheduledEventId },
    api!("/guilds/{}/scheduled-events/{}", guild_id, event_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildScheduledEvents { guild_id: GuildId },
    api!("/guilds/{}/scheduled-events", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildScheduledEventUsers { guild_id: GuildId, event_id: ScheduledEventId },
    api!("/guilds/{}/scheduled-events/{}/users", guild_id, event_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildSticker { guild_id: GuildId, sticker_id: StickerId },
    api!("/guilds/{}/stickers/{}", guild_id, sticker_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildStickers { guild_id: GuildId },
    api!("/guilds/{}/stickers", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildVanityUrl { guild_id: GuildId },
    api!("/guilds/{}/vanity-url", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildVoiceStates { guild_id: GuildId, user_id: UserId },
    api!("/guilds/{}/voice-states/{}", guild_id, user_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildVoiceStateMe { guild_id: GuildId },
    api!("/guilds/{}/voice-states/@me", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildWebhooks { guild_id: GuildId },
    api!("/guilds/{}/webhooks", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildWelcomeScreen { guild_id: GuildId },
    api!("/guilds/{}/welcome-screen", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    GuildThreadsActive { guild_id: GuildId },
    api!("/guilds/{}/threads/active", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id.get().into()));

    Guilds,
    api!("/guilds"),
    Some(RatelimitingKind::Path);

    Invite { code: &'a str },
    api!("/invites/{}", code),
    Some(RatelimitingKind::Path);

    Oauth2ApplicationCurrent,
    api!("/oauth2/applications/@me"),
    None;

    StatusIncidentsUnresolved,
    status!("/incidents/unresolved.json"),
    None;

    StatusMaintenancesActive,
    status!("/scheduled-maintenances/active.json"),
    None;

    StatusMaintenancesUpcoming,
    status!("/scheduled-maintenances/upcoming.json"),
    None;

    Sticker { sticker_id: StickerId },
    api!("/stickers/{}", sticker_id),
    Some(RatelimitingKind::Path);

    StickerPacks,
    api!("/sticker-packs"),
    Some(RatelimitingKind::Path);

    User { user_id: UserId },
    api!("/users/{}", user_id),
    Some(RatelimitingKind::Path);

    UserMe,
    api!("/users/@me"),
    Some(RatelimitingKind::Path);

    UserMeConnections,
    api!("/users/@me/connections"),
    Some(RatelimitingKind::Path);

    UserMeDmChannels,
    api!("/users/@me/channels"),
    Some(RatelimitingKind::Path);

    UserMeGuild { guild_id: GuildId },
    api!("/users/@me/guilds/{}", guild_id),
    Some(RatelimitingKind::Path);

    UserMeGuilds,
    api!("/users/@me/guilds"),
    Some(RatelimitingKind::Path);

    VoiceRegions,
    api!("/voice/regions"),
    Some(RatelimitingKind::Path);

    Webhook { webhook_id: WebhookId },
    api!("/webhooks/{}", webhook_id),
    Some(RatelimitingKind::PathAndId(webhook_id.get().into()));

    WebhookWithToken { webhook_id: WebhookId, token: &'a str },
    api!("/webhooks/{}/{}", webhook_id, token),
    Some(RatelimitingKind::PathAndId(webhook_id.get().into()));

    WebhookMessage { webhook_id: WebhookId, token: &'a str, message_id: MessageId },
    api!("/webhooks/{}/{}/messages/{}", webhook_id, token, message_id),
    Some(RatelimitingKind::PathAndId(webhook_id.get().into()));

    WebhookOriginalInteractionResponse { application_id: ApplicationId, token: &'a str },
    api!("/webhooks/{}/{}/messages/@original", application_id, token),
    Some(RatelimitingKind::PathAndId(application_id.get().into()));

    WebhookFollowupMessage { application_id: ApplicationId, token: &'a str, message_id: MessageId },
    api!("/webhooks/{}/{}/messages/{}", application_id, token, message_id),
    Some(RatelimitingKind::PathAndId(application_id.get().into()));

    WebhookFollowupMessages { application_id: ApplicationId, token: &'a str },
    api!("/webhooks/{}/{}", application_id, token),
    Some(RatelimitingKind::PathAndId(application_id.get().into()));

    InteractionResponse { interaction_id: InteractionId, token: &'a str },
    api!("/interactions/{}/{}/callback", interaction_id, token),
    Some(RatelimitingKind::PathAndId(interaction_id.get().into()));

    Command { application_id: ApplicationId, command_id: CommandId },
    api!("/applications/{}/commands/{}", application_id, command_id),
    Some(RatelimitingKind::PathAndId(application_id.get().into()));

    Commands { application_id: ApplicationId },
    api!("/applications/{}/commands", application_id),
    Some(RatelimitingKind::PathAndId(application_id.get().into()));

    GuildCommand { application_id: ApplicationId, guild_id: GuildId, command_id: CommandId },
    api!("/applications/{}/guilds/{}/commands/{}", application_id, guild_id, command_id),
    Some(RatelimitingKind::PathAndId(application_id.get().into()));

    GuildCommandPermissions { application_id: ApplicationId, guild_id: GuildId, command_id: CommandId },
    api!("/applications/{}/guilds/{}/commands/{}/permissions", application_id, guild_id, command_id),
    Some(RatelimitingKind::PathAndId(application_id.get().into()));

    GuildCommands { application_id: ApplicationId, guild_id: GuildId },
    api!("/applications/{}/guilds/{}/commands", application_id, guild_id),
    Some(RatelimitingKind::PathAndId(application_id.get().into()));

    GuildCommandsPermissions { application_id: ApplicationId, guild_id: GuildId },
    api!("/applications/{}/guilds/{}/commands/permissions", application_id, guild_id),
    Some(RatelimitingKind::PathAndId(application_id.get().into()));

    StageInstances,
    api!("/stage-instances"),
    Some(RatelimitingKind::Path);

    StageInstance { channel_id: ChannelId },
    api!("/stage-instances/{}", channel_id),
    Some(RatelimitingKind::Path);
});
