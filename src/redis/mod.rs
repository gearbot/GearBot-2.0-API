use serde::{Serialize, Deserialize};
use uuid::Uuid;

pub mod redis_link;

#[derive(Debug, Serialize)]
pub struct GearBotRequest {
    pub uuid: Uuid,
    pub request: Request
}


#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    TeamInfo
}

#[derive(Debug, Deserialize, Clone)]
pub struct Reply {
    pub uuid: Uuid,
    pub data: ReplyData
}

#[derive(Debug, Deserialize, Clone)]
pub enum ReplyData {
    Blank,//
    TeamInfo(TeamInfo)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamInfo {
    pub members: Vec<TeamMember>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamMember {
    pub username: String,
    pub discriminator: String,
    pub id: u64,
    pub avatar: String,
    pub socials: TeamSocials,
    pub team: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamSocials {
    pub twitter: Option<String>,
    pub github: Option<String>,
    pub website: Option<String>
}