use crate::redis::redis_link::RedisLink;
use crate::routes::{hello_world, not_found, team_info, ws};
use crate::util::config::ApiConfig;
use crate::util::error::{RequestError, StartupError};
use flexi_logger::{colored_opt_format, Age, Cleanup, Criterion, Duplicate, Logger, Naming};
use hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server};
use log::{error, info};
use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

mod redis;
mod routes;
mod util;

pub struct ApiContext {
    pub config: ApiConfig,
    pub redis_link: RedisLink,
}

#[tokio::main]
async fn main() -> Result<(), StartupError> {
    //init logging
    Logger::with_env_or_str("info")
        .duplicate_to_stdout(Duplicate::Debug)
        .log_to_file()
        .directory("logs")
        .format(colored_opt_format)
        .o_timestamp(true)
        .rotate(
            Criterion::Age(Age::Day),
            Naming::Timestamps,
            Cleanup::KeepLogAndZipFiles(10, 30),
        )
        .start_with_specfile("logconfig.toml")
        .map_err(|_| StartupError::NoLoggingSpec)?;

    //load config file
    let config = ApiConfig::new(&env::var("CONFIG_FILE").unwrap_or("config.toml".to_string()))?;
    info!("Config file loaded!");

    let redis_link = RedisLink::new(&config).await?;
    info!("Redis connection established");

    let port = config.port;
    let api_context = Arc::new(ApiContext { config, redis_link });
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let make_svc = make_service_fn(|_conn| {
        let context = api_context.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                let context = context.clone();
                async move { handle_request(req, context).await }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    log::info!(
        "Startup complete, now listening for requests on port {}",
        port
    );

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
    Ok(())
}

async fn handle_request(
    request: Request<Body>,
    context: Arc<ApiContext>,
) -> Result<Response<Body>, Infallible> {
    let mut reply = if let Some(path_and_query) = request.uri().path_and_query() {
        let full_path = path_and_query.path().to_string();
        let skip = usize::from(full_path.starts_with('/'));
        let parts = full_path
            .split('/')
            .skip(skip)
            .skip_while(|p| *p == "api")
            .collect::<Vec<&str>>();
        let method = Method::from(request.method());
        let response = match (&method, parts.as_slice()) {
            (&Method::GET, ["hello"]) => hello_world().await,
            (&Method::GET, ["team_info"]) => team_info(context).await,
            (&Method::GET, ["ws"]) => ws(context, request).await,
            _ => not_found(),
        };

        let reply = match response {
            Ok(response) => response,
            Err(e) => {
                if let RequestError::Server(e) = &e {
                    error!("{}", e)
                }
                Response::builder()
                    .status(e.get_status())
                    .body(Body::from(format!("{}", e)))
                    .unwrap()
            }
        };

        info!("{} {} => {}", method, full_path, reply.status());

        reply
    } else {
        Response::new("how the hell did we get here?".into())
    };

    reply
        .headers_mut()
        .append(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());

    Ok(reply)
}
