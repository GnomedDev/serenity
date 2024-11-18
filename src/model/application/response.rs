#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct InteractionResponse {
    interaction: InteractionCallback,
    resource: Option<InteractionResource>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct InteractionCallback {
    id: InteractionId,
    kind: InteractionType,
    activity_instance_id: Option<FixedString>,
    response_message_id: Option<MessageId>,
    response_message_loading: Option<bool>,
    response_message_ephemeral: Option<bool>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct InteractionResource {
    #[serde(rename = "type")]
    kind: InteractionCallbackType,
    activity_instance: Option<ActivityInstance>,
    message: Option<Message>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ActivityInstance {
    id: FixedString,
}

enum_number! {
    /// An enum representing the interaction callback types.
    ///
    /// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-response-object-interaction-callback-type).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[non_exhaustive]
    pub enum InteractionCallbackType {
        Pong = 0,
        ChannelMessageWithSource = 4,
        DeferredChannelMessageWithSource = 5,
        DeferredUpdateMessage = 6,
        UpdateMessage = 7,
        ApplicationCommandAutocompleteResult = 8,
        Modal = 9,
        PremiumRequired = 10,
        LaunchActivity = 12,
        _ => Unknown(u8),
    }
}
