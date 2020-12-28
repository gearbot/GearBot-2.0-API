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
pub enum WSOutbound {
    Welcome,
    GuildList(UserGuildList),
}

#[derive(Debug, Serialize, Clone)]
pub struct UserGuildList {
    pub gearbot_servers: Vec<MinimalGuild>,
    pub available_servers: Vec<MinimalGuild>
}

#[derive(Debug, Serialize, Clone)]
pub struct MinimalGuild {
    pub id: String, //u64 but String cause js 😒
    pub name: String,
    #[serde(skip_serializing_if = "is_default")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub owned: bool,
    #[serde(skip_serializing_if = "is_default")]
    pub permissions: u64,
}

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}