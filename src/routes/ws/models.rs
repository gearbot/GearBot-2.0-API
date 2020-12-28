use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum WSRequest {
    Identify {
        token: String
    },
    GuildList,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
pub struct Reply {
    pub uuid: Uuid,
    pub data: ReplyData,
}

#[derive(Debug, Serialize, Clone)]
pub enum ReplyData {
    GuildList(UserGuildList)
}

#[derive(Debug, Serialize, Clone)]
pub struct UserGuildList {
    gearbot_servers: Vec<MinimalGuild>,
    available_servers: Vec<MinimalGuild>
}

#[derive(Debug, Serialize, Clone)]
pub struct MinimalGuild {

}