use url::Url;

use crate::model::gateway::{Activity, ActivityType};
use crate::model::user::OnlineStatus;

/// Presence data of the current user.
#[derive(Clone, Debug, Default)]
pub struct PresenceBuilder {
    pub(crate) activity: Option<ActivityBuilder>,
    pub(crate) status: OnlineStatus,
}

/// Activity data of the current user.
#[derive(Clone, Debug, Serialize)]
pub struct ActivityBuilder {
    name: String,
    #[serde(rename = "type")]
    kind: ActivityType,
    state: Option<String>,
    url: Option<Url>,
}

impl ActivityBuilder {
    /// Creates an activity that appears as `Playing <name>`.
    #[must_use]
    pub fn playing(name: impl Into<String>) -> Self {
        Self {
            name: name.into().into(),
            kind: ActivityType::Playing,
            state: None,
            url: None,
        }
    }

    /// Creates an activity that appears as `Streaming <name>`.
    pub fn streaming(name: impl Into<String>, url: Url) -> Self {
        Self {
            name: name.into().into(),
            kind: ActivityType::Streaming,
            state: None,
            url: Some(url),
        }
    }

    /// Creates an activity that appears as `Listening to <name>`.
    #[must_use]
    pub fn listening(name: impl Into<String>) -> Self {
        Self {
            name: name.into().into(),
            kind: ActivityType::Listening,
            state: None,
            url: None,
        }
    }

    /// Creates an activity that appears as `Watching <name>`.
    #[must_use]
    pub fn watching(name: impl Into<String>) -> Self {
        Self {
            name: name.into().into(),
            kind: ActivityType::Watching,
            state: None,
            url: None,
        }
    }

    /// Creates an activity that appears as `Competing in <name>`.
    #[must_use]
    pub fn competing(name: impl Into<String>) -> Self {
        Self {
            name: name.into().into(),
            kind: ActivityType::Competing,
            state: None,
            url: None,
        }
    }

    /// Creates an activity that appears as `<state>`.
    #[must_use]
    pub fn custom(state: impl Into<String>) -> Self {
        Self {
            // discord seems to require a name for custom activities
            // even though it's not displayed
            name: "~".to_string().into(),
            kind: ActivityType::Custom,
            state: Some(state.into().into()),
            url: None,
        }
    }
}

impl From<Activity> for ActivityBuilder {
    fn from(activity: Activity) -> Self {
        Self {
            name: activity.name.into(),
            kind: activity.kind,
            state: activity.state.map(Into::into),
            url: activity.url,
        }
    }
}
