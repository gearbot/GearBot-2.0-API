mod hello;
pub use hello::hello_world;

mod team;
pub use team::team_info;

mod ws;
pub use ws::ws;

pub mod discord;

use crate::error::RequestError;
use hyper::{Body, Response, StatusCode};

pub fn not_found() -> Result<Response<Body>, RequestError> {
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())?)
}
