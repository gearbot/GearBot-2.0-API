use crate::error::RequestError;
use crate::ApiContext;
use hyper::{Body, Response, Request, StatusCode};
use std::sync::Arc;
use hyper::header::COOKIE;
use crate::util::get_user_id;


pub async fn user_info(ctx: Arc<ApiContext>, request: Request<Body>) -> Result<Response<Body>, RequestError> {
    if let Some(user_id) = get_user_id(&ctx, &request).await? {
        //welcome authenticated user!
        if let Some(user_info) = ctx.redis_link.get_user_info(user_id).await? {
            return Ok(
                Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(serde_json::to_string(&user_info).unwrap()))?
            )
        }
    }
        Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::empty())?)
}
