use crate::error::RequestError;
use hyper::{Body, Response};

pub async fn hello_world() -> Result<Response<Body>, RequestError> {
    Ok(Response::new("Hello, World".into()))
}
