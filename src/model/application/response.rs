use small_fixed_array::FixedString;

use super::InteractionType;
use crate::model::channel::Message;
use crate::model::id::{InteractionId, MessageId};

/// The representation of a response to an interaction.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]

pub struct InteractionResponse {
    /// The interaction object associated with the interaction response.
    pub interaction: InteractionCallback,
    /// The resource that was created by the interaction response.
    pub resource: Option<InteractionResource>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct InteractionCallback {
    /// The ID of the interaction responded to.
    pub id: InteractionId,
    /// The type of interaction responded to.
    pub kind: InteractionType,
    /// The Instance ID of the Activity if one was launched or joined.  
    pub activity_instance_id: Option<FixedString>,
    /// The Instance ID of the Activity if one was launched or joined.  
    pub response_message_id: Option<MessageId>,
    pub response_message_loading: Option<bool>,
    pub response_message_ephemeral: Option<bool>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct InteractionResource {
    #[serde(rename = "type")]
    pub kind: InteractionCallbackType,
    pub activity_instance: Option<ActivityInstance>,
    pub message: Option<Message>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct ActivityInstance {
    pub id: FixedString,
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
