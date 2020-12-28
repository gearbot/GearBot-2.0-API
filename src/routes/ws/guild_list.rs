use std::sync::Arc;
use crate::ApiContext;
use crate::error::WSMessageError;
use tokio_tungstenite::tungstenite::Message;
use serde::Serialize;
use crate::routes::ws::models::{WSOutbound, UserGuildList, MinimalGuild};
use crate::util::get_user_guilds;

pub async fn guild_list(ctx: &Arc<ApiContext>, user_id: u64) -> Result<WSOutbound, WSMessageError> {
    if let Some(token) = ctx.redis_link.get(&format!("access_token:{}", user_id)).await? {
        // all guilds the user is in
        let discord_list_handle = tokio::spawn(get_user_guilds(ctx.clone(), user_id, token));
        //request mutual servers from the bot
        let bot_list = ctx.redis_link.get_mutual_guilds(user_id).await?;

        let discord_list = discord_list_handle.await.unwrap()?;

        let bot_guilds = bot_list.iter().map(|guild| guild.id).collect::<Vec<u64>>();

        //TODO: filter gearbot permissions to see the guild?
        let gearbot_servers = bot_list.iter().filter(|guild|true).map(|guild| MinimalGuild {
            id: guild.id.to_string(),
            name: guild.name.clone(),
            icon: guild.icon.clone(),
            owned: guild.owned,
            permissions: guild.permissions
        })
            .collect();

        let mut available_servers = vec![];


        for guild in discord_list {
            if !bot_guilds.contains(&guild.id.0) {
                //TODO: check for manage server perm
                available_servers.push(MinimalGuild {
                    id: guild.id.to_string(),
                    name: guild.name.clone(),
                    icon: guild.icon.clone(),
                    owned: guild.owner,
                    permissions: 0
                })
            }
        }


        Ok(WSOutbound::GuildList(UserGuildList {
            gearbot_servers,
            available_servers: vec![]
        }))
    } else {
        //TODO: try refresh token
        Err(WSMessageError::NoValidDiscordAuthToken)
    }

}