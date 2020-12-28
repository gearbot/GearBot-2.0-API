use twilight_model::id::GuildId;
use twilight_model::guild::Permissions;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct UserGuild {
    pub id: GuildId,
    pub name: String,
    pub icon: Option<String>,
    pub owner: bool,
    pub permissions: Permissions,
    pub features: Vec<String>
}