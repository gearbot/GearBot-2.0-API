use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod redis_link;

#[derive(Debug, Serialize)]
pub struct GearBotRequest {
    pub uuid: Uuid,
    pub request: Request,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    TeamInfo,
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

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}
