use hyper::StatusCode;
use std::fmt;
use tokio::sync::oneshot::error::RecvError;
use std::fmt::Formatter;
use std::borrow::Cow;

#[derive(Debug)]
pub enum StartupError {
    NoConfig,
    InvalidConfig,
    NoLoggingSpec,
    DarkRedis(darkredis::Error),
}

#[derive(Debug)]
pub enum RequestError {
    Server(ServerError),
    BadRequest(BadRequestError),
    NotFound,
    Forbidden,
}

#[derive(Debug)]
pub enum CommunicationError {
    Timeout,
    Receiver(RecvError),
    DarkRedis(darkredis::Error),
    WrongReplyType,
    DataFormat(serde_json::Error),
}

#[derive(Debug)]
pub enum ServerError {
    HyperHttp(hyper::http::Error),
    Communication(CommunicationError),
    Hyper(hyper::Error),
    DiscordError(String),
    Database(DatabaseError),
}

#[derive(Debug)]
pub enum BadRequestError {
    UpgradeOnly,
    MissingWsKey,
    NoAccessCode,
}

#[derive(Debug)]
pub enum DatabaseError {
    Sqlx(sqlx::Error),
    Deserializing(serde_json::Error),
    Serializing(serde_json::Error),
    Darkredis(darkredis::Error),
}

#[derive(Debug)]
pub enum WSMessageError {
    Tungstenite(tokio_tungstenite::tungstenite::Error),
    Database(DatabaseError),
    Communication(CommunicationError),
    CorruptMessage(serde_json::Error),
    NotAuthorized,
    BadAuthorization,
    AlreadyAuthorized,
    ClosedGracefully,
    NoValidDiscordAuthToken,
    DiscordRequest(RequestError)
}


impl fmt::Display for WSMessageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            WSMessageError::Tungstenite(e) => write!(f, "Tunstenite failure: {}", e),
            WSMessageError::Database(e) => write!(f, "Failed to fetch database information: {}", e),
            WSMessageError::Communication(e) => write!(f, "Failed to communicate with GearBot: {}", e),
            WSMessageError::CorruptMessage(e) => write!(f, "Corrupt message recieved: {}", e),
            WSMessageError::NotAuthorized => write!(f, "Someone didn't identify themselves"),
            WSMessageError::BadAuthorization => write!(f, "Someone gave us an invalid token to try and identify"),
            WSMessageError::AlreadyAuthorized => write!(f, "Someone double identified"),
            WSMessageError::ClosedGracefully => write!(f, "Connection closed by client"),
            WSMessageError::NoValidDiscordAuthToken => write!(f, "No valid discord oauth2 token found"),
            WSMessageError::DiscordRequest(e) => write!(f, "Failed to fetch information from the discord api: {}", e)
        }
    }
}

const CORRUPT_MESSAGE: &str = "Corrupt message recieved";
const NOT_AUTHORIZED: &str = "You failed to identify yourself first, access denied!";
const BAD_AUTHORIZATION: &str = "You failed to identify yourself first, access denied!";
const ALREADY_AUTHORIZED: &str = "You can not identify twice!";
const TUNGSTENITE: &str = "Unable to process message";
const NO_VALID_DISCORD_AUTH: &str = "No valid discord oauth token was found in storage for this user";

impl WSMessageError {
    pub fn closes_socket(&self) -> bool {
        match self {
            WSMessageError::Tungstenite(_) |
            WSMessageError::CorruptMessage(_) |
            WSMessageError::NotAuthorized |
            WSMessageError::AlreadyAuthorized |
            WSMessageError::BadAuthorization |
            WSMessageError::NoValidDiscordAuthToken => true,
            _ => false
        }
    }

    pub fn get_close_message(&self) -> &'static str {
        match self {
            WSMessageError::CorruptMessage(_) => CORRUPT_MESSAGE,
            WSMessageError::NotAuthorized => NOT_AUTHORIZED,
            WSMessageError::BadAuthorization => BAD_AUTHORIZATION,
            WSMessageError::AlreadyAuthorized => ALREADY_AUTHORIZED,
            WSMessageError::Tungstenite(_) => TUNGSTENITE,
            WSMessageError::NoValidDiscordAuthToken => NO_VALID_DISCORD_AUTH,
            _ => unreachable!()
        }
    }
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::Sqlx(e) => write!(f, "Database failure: {:?}", e),
            DatabaseError::Deserializing(e) => write!(f, "Failed to deserialize: {}", e),
            DatabaseError::Serializing(e) => write!(f, "Failed to seralize: {}", e),
            DatabaseError::Darkredis(e) => write!(f, "Redis failure: {}", e),
        }
    }
}

impl From<darkredis::Error> for DatabaseError {
    fn from(e: darkredis::Error) -> Self {
        DatabaseError::Darkredis(e)
    }
}

impl From<sqlx::Error> for DatabaseError {
    fn from(e: sqlx::Error) -> Self {
        DatabaseError::Sqlx(e)
    }
}

impl RequestError {
    pub fn get_status(&self) -> StatusCode {
        match self {
            RequestError::Server(_) => StatusCode::INTERNAL_SERVER_ERROR,
            RequestError::BadRequest(_) => StatusCode::BAD_REQUEST,
            RequestError::NotFound => StatusCode::NOT_FOUND,
            RequestError::Forbidden => StatusCode::FORBIDDEN,
        }
    }
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestError::Server(_) => write!(f, "Internal server error!"),
            RequestError::BadRequest(e) => write!(f, "Bad request! {}", e),
            RequestError::NotFound => write!(f, "Unknown route"),
            RequestError::Forbidden => write!(f, "Access denied"),
        }
    }
}

impl fmt::Display for StartupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StartupError::NoConfig => write!(f, "No config found"),
            StartupError::InvalidConfig => write!(f, "Config file is not valid"),
            StartupError::NoLoggingSpec => write!(f, "Unable to load log spec file"),
            StartupError::DarkRedis(e) => write!(f, "Error creating the redis pool: {}", e),
        }
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::HyperHttp(e) => write!(f, "Error assembling hyper response: {}", e),
            ServerError::Communication(e) => write!(f, "Error communicating with GearBot: {}", e),
            ServerError::Hyper(e) => write!(f, "Error making a request to the discord api: {}", e),
            ServerError::Database(e) => write!(f, "Database error occured: {}", e),
            ServerError::DiscordError(e) => write!(f, "Error making a request to discord: {}", e)
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
            CommunicationError::Timeout => write!(f, "GearBot did not respond in time"),
            CommunicationError::Receiver(e) => {
                write!(f, "Error receiving reply from GearBot: {}", e)
            }
            CommunicationError::DarkRedis(e) => {
                write!(f, "Error pushing the message to redis: {}", e)
            }
            CommunicationError::WrongReplyType => {
                write!(f, "Received wrong reply data type for the requested data")
            }
            CommunicationError::DataFormat(e) => write!(f, "JSON was in an unexpected form: {}", e),
        }
    }
}

impl From<hyper::http::Error> for RequestError {
    fn from(e: hyper::http::Error) -> Self {
        RequestError::Server(ServerError::HyperHttp(e))
    }
}

impl From<darkredis::Error> for StartupError {
    fn from(e: darkredis::Error) -> Self {
        StartupError::DarkRedis(e)
    }
}

impl From<RecvError> for CommunicationError {
    fn from(e: RecvError) -> Self {
        CommunicationError::Receiver(e)
    }
}

impl From<darkredis::Error> for CommunicationError {
    fn from(e: darkredis::Error) -> Self {
        CommunicationError::DarkRedis(e)
    }
}

impl From<CommunicationError> for RequestError {
    fn from(e: CommunicationError) -> Self {
        RequestError::Server(ServerError::Communication(e))
    }
}

impl From<BadRequestError> for RequestError {
    fn from(e: BadRequestError) -> Self {
        RequestError::BadRequest(e)
    }
}

impl From<hyper::Error> for RequestError {
    fn from(e: hyper::Error) -> Self {
        RequestError::Server(ServerError::Hyper(e))
    }
}

impl From<DatabaseError> for RequestError {
    fn from(e: DatabaseError) -> Self {
        RequestError::Server(ServerError::Database(e))
    }
}

impl From<DatabaseError> for WSMessageError {
    fn from(e: DatabaseError) -> Self {
        WSMessageError::Database(e)
    }
}

impl From<CommunicationError> for WSMessageError {
    fn from(e: CommunicationError) -> Self {
        WSMessageError::Communication(e)
    }
}
impl From<RequestError> for WSMessageError {
    fn from(e: RequestError) -> Self {
        WSMessageError::DiscordRequest(e)
    }
}