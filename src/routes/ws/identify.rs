use crate::error::WSMessageError;
use crate::ApiContext;
use crate::redis::UserInfo;
use std::sync::Arc;

pub async fn identify(ctx: &Arc<ApiContext>, token: &str) -> Result<(u64, UserInfo), WSMessageError> {
    if let Some(user_id) = ctx.redis_link.get::<u64>(&format!("dash_token:{}", token)).await? {
        if let Some(user_info) = ctx.redis_link.get_user_info(user_id).await? {
            return Ok(
                (user_id, user_info)
            );
        }
    }
    return Err(WSMessageError::BadAuthorization);
}