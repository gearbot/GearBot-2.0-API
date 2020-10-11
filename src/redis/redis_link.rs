use crate::util::config::ApiConfig;
use crate::util::error::{StartupError, CommunicationError};
use darkredis::{ConnectionPool, Connection};
use uuid::Uuid;
use tokio::sync::broadcast::{Receiver, channel, Sender};
use crate::redis::{Reply, Request, GearBotRequest, TeamInfo, ReplyData};
use tokio::time::{timeout, Duration};
use futures_util::StreamExt;

pub struct RedisLink {
    pool: ConnectionPool,
    sender: Sender<Reply>,
}

impl RedisLink {
    pub async fn new(config: &ApiConfig) -> Result<Self, StartupError> {
        let pool = darkredis::ConnectionPool::create(config.redis.to_string(), None, 5).await?;
        let (sender, _) = channel(5);
        let connection = pool.spawn("api_connection").await?;
        let s = sender.clone();
        tokio::spawn(async move {
            establish_bot_link(s, connection).await;
        });

        Ok(Self {
            pool,
            sender,
        })
    }

    pub async fn get_team_members(&self) -> Result<TeamInfo, CommunicationError> {
        if let ReplyData::TeamInfo(info) = self.get_reply(Request::TeamInfo, Some(5)).await?.data {
            Ok(info)
        } else {
            Err(CommunicationError::WrongReplyType)
        }
    }

    async fn get_reply(&self, request: Request, max_wait: Option<u64>) -> Result<Reply, CommunicationError> {
        let max_wait = max_wait.unwrap_or(60);
        let uuid = Uuid::new_v4();

        let request = GearBotRequest {
            uuid,
            request,
        };

        //scope for redis connection
        {
            let mut connection = self.pool.get().await;
            connection.publish("api-out", serde_json::to_string(&request).unwrap()).await?;
        }


        timeout(Duration::from_secs(max_wait), self.await_reply(uuid)).await
            .map_err(|_| CommunicationError::TimeoutError)
    }
    async fn await_reply(&self, uuid: Uuid) -> Reply {
        while let Ok(reply) = self.sender.subscribe().recv().await {
            if reply.uuid == uuid {
                return reply
            }
        }
        unreachable!()
    }
}

async fn establish_bot_link(sender: Sender<Reply>, connection: Connection) {
    log::debug!("establishing api connection");
    connection.subscribe(&["gearbot-out"])
        .await
        .unwrap()
        .for_each(|message| async {
            let m = message;
            let reply: Reply = serde_json::from_slice(&m.message).unwrap();
            sender.send(reply);
        }).await
}

