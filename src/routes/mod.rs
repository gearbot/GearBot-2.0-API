mod hello;
pub use hello::hello_world;

mod team;
pub use team::team_info;

use hyper::{Body, Response, StatusCode};
use crate::util::error::RequestError;

pub fn not_found() -> Result<Response<Body>, RequestError> {
    Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty())?)
}