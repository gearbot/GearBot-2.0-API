use crate::redis::{GearBotRequest, Reply, ReplyData, Request, TeamInfo};
use crate::util::config::ApiConfig;
use crate::util::error::{CommunicationError, StartupError};
use darkredis::{Connection, ConnectionPool};
use futures_util::StreamExt;
use tokio::sync::broadcast;
use tokio::time::{timeout, Duration};
use uuid::Uuid;

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
}

async fn establish_bot_link(sender: broadcast::Sender<Reply>, connection: Connection) {
    log::debug!("establishing api connection");
    connection
        .subscribe(&["gearbot-out"])
        .await
        .unwrap()
        .for_each(|message| async {
            let m = message;
            // TODO: Get `TryStreamExt` working here after https://github.com/Bunogi/darkredis/pull/19
            // is merged and released.
            let reply: Reply = serde_json::from_slice(&m.message).unwrap();

            sender.send(reply);
        })
        .await
}
