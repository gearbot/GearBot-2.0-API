use crate::config::ApiConfig;
use crate::error::{CommunicationError, StartupError, DatabaseError};
use crate::redis::{GearBotRequest, Reply, ReplyData, Request, TeamInfo, UserInfo, MinimalGuildInfo};
use darkredis::{Connection, ConnectionPool};
use futures_util::StreamExt;
use tokio::sync::broadcast;
use tokio::time::{timeout, Duration};
use uuid::Uuid;
use serde::de::DeserializeOwned;
use serde::Serialize;
use twilight_model::id::UserId;

pub struct RedisLink {
    pool: ConnectionPool,
    sender: broadcast::Sender<Reply>,
}

impl RedisLink {
    pub async fn new(config: &ApiConfig) -> Result<Self, StartupError> {
        let pool = darkredis::ConnectionPool::create(config.redis.to_string(), None, 5).await?;
        let (sender, _) = broadcast::channel(5);
        let connection = pool.spawn("api_connection").await?;
        let s = sender.clone();
        tokio::spawn(async move {
            establish_bot_link(s, connection).await;
        });

        Ok(Self { pool, sender })
    }

    pub async fn get_team_members(&self) -> Result<TeamInfo, CommunicationError> {
        if let ReplyData::TeamInfo(info) = self.get_reply(Request::TeamInfo, Some(5)).await?.data {
            Ok(info)
        } else {
            Err(CommunicationError::WrongReplyType)
        }
    }

    pub async fn get_user_info(&self, user_id: u64) -> Result<Option<UserInfo>, CommunicationError> {
        if let ReplyData::UserInfo(info) = self.get_reply(Request::UserInfo(user_id), Some(60)).await?.data {
            Ok(info)
        } else {
            Err(CommunicationError::WrongReplyType)
        }
    }

    pub async fn get_mutual_guilds(&self, user_id: u64) -> Result<Vec<MinimalGuildInfo>, CommunicationError> {
        if let ReplyData::MutualGuildList(info) = self.get_reply(Request::MutualGuilds(user_id), Some(60)).await?.data {
            Ok(info)
        } else {
            Err(CommunicationError::WrongReplyType)
        }
    }

    async fn get_reply(
        &self,
        request: Request,
        max_wait: Option<u64>,
    ) -> Result<Reply, CommunicationError> {
        let max_wait = max_wait.unwrap_or(60);
        let uuid = Uuid::new_v4();

        let request = GearBotRequest { uuid, request };

        //scope for redis connection
        {
            let mut connection = self.pool.get().await;
            let message = serde_json::to_vec(&request).map_err(CommunicationError::DataFormat)?;
            connection.publish("api-out", message).await?;
        }

        timeout(Duration::from_secs(max_wait), self.await_reply(uuid))
            .await
            .map_err(|_| CommunicationError::Timeout)
    }
    async fn await_reply(&self, uuid: Uuid) -> Reply {
        while let Ok(reply) = self.sender.subscribe().recv().await {
            if reply.uuid == uuid {
                return reply;
            }
        }
        unreachable!()
    }

    /// Retrieves a value from Redis.
    ///
    /// Returns `None` if the key didn't exist.
    pub async fn get<D: DeserializeOwned>(&self, key: &str) -> Result<Option<D>, DatabaseError> {
        let mut conn = self.pool.get().await;

        if let Some(value) = conn.get(key).await? {
            let value = serde_json::from_slice(&value).map_err(DatabaseError::Deserializing)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Inserts a value into Redis.
    ///
    /// The value will automatically expire at the optionally provided time.
    pub async fn set<T: Serialize>(&self, key: &str, value: &T, expiry: Option<u32>) -> Result<(), DatabaseError> {
        let mut conn = self.pool.get().await;

        let data = serde_json::to_string(value).map_err(DatabaseError::Serializing)?;

        match expiry {
            Some(ttl) => conn.set_and_expire_seconds(key, data, ttl).await?,
            None => conn.set(key, data).await?,
        }

        Ok(())
    }

    /// Deletes a value from Redis.
    pub async fn delete(&self, key: &str) -> Result<(), darkredis::Error> {
        let mut conn = self.pool.get().await;

        conn.del(key).await?;

        Ok(())
    }
}

async fn establish_bot_link(sender: broadcast::Sender<Reply>, connection: Connection) {
    log::debug!("establishing api connection");
    connection
        .subscribe(&["gearbot-out"])
        .await
        .unwrap()
        .for_each(|message| async {
            let m = message;
            log::debug!("{}", String::from_utf8(m.message.clone()).unwrap());
            match serde_json::from_slice(&m.message)  {
                Ok(reply) =>
                    {
                        sender.send(reply);
                    },
                Err(e) => {log::error!("{}", e);}
            }

            ;
        })
        .await
}
