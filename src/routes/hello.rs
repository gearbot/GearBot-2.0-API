use hyper::{Body, Response};
use crate::util::error::RequestError;

pub async fn hello_world() -> Result<Response<Body>, RequestError> {
    Ok(Response::new("Hello, World".into()))
}