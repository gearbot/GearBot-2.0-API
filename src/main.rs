use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server, Method};
use hyper::service::{make_service_fn, service_fn};
use crate::routes::{hello_world, not_found, team_info};
use log::{info, error};
use crate::util::error::{StartupError, RequestError};
use flexi_logger::{Logger, Duplicate, Criterion, Naming, Age, Cleanup, colored_opt_format};
use crate::util::config::ApiConfig;
use std::env;
use crate::redis::redis_link::RedisLink;
use std::sync::Arc;
use hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN;
use hyper::http::HeaderValue;

mod routes;
mod util;
mod redis;

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
    let api_context = Arc::new(ApiContext {
        config,
        redis_link,
    });
    let c = api_context.clone();
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let make_svc = make_service_fn(|_conn| {
        let context = api_context.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                let context = context.clone();
                async move {
                    handle_request(req, context).await
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    log::info!("Startup complete, now listening for requests on port {}", port);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
    Ok(())
}

async fn handle_request(request: Request<Body>, context: Arc<ApiContext>) -> Result<Response<Body>, Infallible> {
    let (request_parts, _body) = request.into_parts();


    let mut reply = if let Some(path_and_query) = request_parts.uri.path_and_query() {
        let full_path = path_and_query.path();
        let skip = usize::from(full_path.starts_with('/'));
        let parts = full_path.split('/').skip(skip).skip_while(|p| *p == "api").collect::<Vec<&str>>();

        let response = match (&request_parts.method, parts.as_slice()) {
            (&Method::GET, ["hello"]) => hello_world().await,
            (&Method::GET, ["team_info"]) => team_info(context).await,
            _ => not_found()
        };


        let reply = match response {
            Ok(response) => response,
            Err(e) => {
                if let RequestError::Server(e) = &e {
                    error!("{}", e)
                }
                Response::builder().status(e.get_status()).body(Body::from(format!("{}", e))).unwrap()
            }
        };

        info!("{} {} => {}", request_parts.method, full_path, reply.status());

        reply
    } else {
        Response::new("how the hell did we get here?".into())
    };

    reply.headers_mut().append(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());

    Ok(reply)
}