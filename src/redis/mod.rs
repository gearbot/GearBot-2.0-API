use serde::{Deserialize, Serialize};
use uuid::Uuid;
use twilight_model::user::UserFlags;

pub mod redis_link;

#[derive(Debug, Serialize)]
pub struct GearBotRequest {
    pub uuid: Uuid,
    pub request: Request,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    TeamInfo,
    UserInfo(u64)
}

#[derive(Debug, Deserialize, Clone)]
pub struct Reply {
    pub uuid: Uuid,
    pub data: ReplyData,
}

#[derive(Debug, Deserialize, Clone)]
pub enum ReplyData {
    Blank, //
    TeamInfo(TeamInfo),
    UserInfo(Option<UserInfo>)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamInfo {
    pub members: Vec<TeamMember>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamMember {
    pub username: String,
    pub discriminator: String,
    pub id: String, //string to accomodate javascript
    pub avatar: String,
    pub socials: TeamSocials,
    pub team: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamSocials {
    #[serde(skip_serializing_if = "is_default")]
    pub twitter: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub github: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub website: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub name: String,
    pub discriminator: String,
    #[serde(default, skip_serializing_if = "is_default")]
    pub avatar: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub bot_user: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub system_user: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub public_flags: Option<UserFlags>,
}

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}
