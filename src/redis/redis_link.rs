use crate::util::config::ApiConfig;
use crate::util::error::{StartupError, CommunicationError};
use darkredis::ConnectionPool;
use std::collections::HashMap;
use uuid::Uuid;
use tokio::sync::RwLock;
use tokio::sync::oneshot::{Receiver, channel, Sender};
use crate::redis::{Reply, Request, GearBotRequest, TeamInfo, ReplyData};
use tokio::time::{timeout, Duration};
use futures_util::StreamExt;

pub struct RedisLink {
    pool: ConnectionPool,
    pending: RwLock<HashMap<Uuid, Sender<Reply>>>,
}

impl RedisLink {
    pub async fn new(config: &ApiConfig) -> Result<Self, StartupError> {
        let pool = darkredis::ConnectionPool::create(config.redis.to_string(), None, 5).await?;
        Ok(Self {
            pool,
            pending: RwLock::new(HashMap::new()),
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

        let (sender, receiver) = channel();
        //scope for write lock
        {
            let mut pending = self.pending.write().await;
            pending.insert(uuid, sender);
        }

        //scope for redis connection
        {
            let mut connection = self.pool.get().await;
            connection.publish("api-out", serde_json::to_string(&request).unwrap()).await?;
        }

        timeout(Duration::from_secs(max_wait), receiver).await
            .map_err(|_| CommunicationError::TimeoutError)?
            .map_err(|e| CommunicationError::ReceiverError(e))
    }

    pub async fn establish_bot_link(&self) {
        let con = match self.pool.spawn("api_connection").await {
            Ok(con) => con,
            Err(e) => {
                log::error!("ERROR: {}", e);
                panic!("error");
            }
        };
        log::debug!("establishing api connection");
        con.subscribe(&["gearbot-out"])
            .await
            .unwrap()
            .for_each(|message| async move{
                let reply: Reply = serde_json::from_slice(&message.message).unwrap();
                let mut pending = self.pending.write().await;
                if let Some(sender) = pending.remove(&reply.uuid) {
                    sender.send(reply);
                } else {
                    log::error!("Got a reply with uuid {} but no receiver was pending for it", reply.uuid);
                }
            }).await;
    }
}