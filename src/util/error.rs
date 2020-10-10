use std::error;
use std::fmt;
use hyper::StatusCode;
use tokio::sync::oneshot::error::RecvError;

#[derive(Debug)]
pub enum StartupError {
    NoConfig,
    InvalidConfig,
    NoLoggingSpec,
    DarkRedis(darkredis::Error)
}

#[derive(Debug)]
pub enum RequestError {
    Server(ServerError),
    BadRequest(BadRequestError),
    NotFound,
    Forbidden
}

#[derive(Debug)]
pub enum CommunicationError {
    TimeoutError,
    ReceiverError(RecvError),
    DarkRedisError(darkredis::Error),
    WrongReplyType
}

#[derive(Debug)]
pub enum ServerError {
    HyperError(hyper::http::Error),
    CommunicationError(CommunicationError)
}

#[derive(Debug)]
pub enum BadRequestError {

}

impl RequestError {
    pub fn get_status(&self) -> StatusCode {
        match self {
            RequestError::Server(_) => StatusCode::INTERNAL_SERVER_ERROR,
            RequestError::BadRequest(_) => StatusCode::BAD_REQUEST,
            RequestError::NotFound => StatusCode::NOT_FOUND,
            RequestError::Forbidden => StatusCode::FORBIDDEN
        }
    }
}


impl error::Error for StartupError {}
impl error::Error for RequestError {}
impl error::Error for CommunicationError {}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestError::Server(_) => write!(f, "Internal server error!"),
            RequestError::BadRequest(e) => write!(f, "Bad request! {}", e),
            RequestError::NotFound => write!(f, "Unknown route"),
            RequestError::Forbidden => write!(f, "Access denied")
        }
    }
}

impl fmt::Display for StartupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StartupError::NoConfig => write!(f, "No config found"),
            StartupError::InvalidConfig => write!(f, "Config file is not valid"),
            StartupError::NoLoggingSpec => write!(f, "Unable to load log spec file"),
            StartupError::DarkRedis(e) => write!(f, "Error creating the redis pool: {}", e)
        }
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::HyperError(e) => write!(f, "Error assembling hyper response: {}", e),
            ServerError::CommunicationError(e) => write!(f, "Error communicating with GearBot: {}", e),
        }
    }
}

impl fmt::Display for BadRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "placeholder")
    }
}

impl fmt::Display for CommunicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommunicationError::TimeoutError => write!(f, "GearBot did not respond in time"),
            CommunicationError::ReceiverError(e) => write!(f, "Error receiving reply from GearBot: {}", e),
            CommunicationError::DarkRedisError(e) => write!(f, "Error pushing the message to redis: {}", e),
            CommunicationError::WrongReplyType => write!(f, "Received wrong reply data type for the requested data")
        }
    }
}

impl From<hyper::http::Error> for RequestError {
    fn from(e: hyper::http::Error) -> Self {
        RequestError::Server(ServerError::HyperError(e))
    }
}

impl From<darkredis::Error> for StartupError {
    fn from (e: darkredis::Error) -> Self {
        StartupError::DarkRedis(e)
    }
}

impl From<RecvError> for CommunicationError {
    fn from(e: RecvError) -> Self {
        CommunicationError::ReceiverError(e)
    }
}

impl From<darkredis::Error> for CommunicationError {
    fn from (e: darkredis::Error) -> Self {
        CommunicationError::DarkRedisError(e)
    }
}

impl From<CommunicationError> for RequestError {
    fn from(e: CommunicationError) -> Self {
        RequestError::Server(ServerError::CommunicationError(e))
    }
}