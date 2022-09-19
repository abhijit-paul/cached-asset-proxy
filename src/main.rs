use cached_asset_proxy::filters;
use log::info;
use r2d2_redis::r2d2;
use r2d2_redis::RedisConnectionManager;
use std::net::SocketAddr;
use std::time;
use warp::Filter;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    env_logger::init();

    let config = cached_asset_proxy::Config::new().expect("Failed to load config");
    info!("{:?}", config);

    let redis_host = config.clone().redis_host;

    let redis_conn_manager = RedisConnectionManager::new(redis_host).unwrap();

    let redis_pool = r2d2::Pool::builder()
        .max_size(10)
        .connection_timeout(time::Duration::from_millis(3000))
        .build_unchecked(redis_conn_manager);

    let origins: Vec<&str> = config.allow_origins.split(',').collect();
    let cors = warp::cors()
        .allow_origins(origins)
        .allow_methods(vec!["GET", "OPTIONS", "HEAD"]);

    let client = reqwest::Client::builder()
        .gzip(true)
        .build()
        .expect("could not build http client");

    let routes = filters::alive()
        .or(filters::ready())
        .or(filters::assetmap(
            config.clone(),
            client.clone(),
            redis_pool.clone(),
        ))
        .or(filters::asset(
            config.clone(),
            client.clone(),
            redis_pool.clone(),
        ))
        .recover(cached_asset_proxy::customize_error)
        .with(cors)
        .with(warp::log("cached-asset-proxy"));

    info!("Host:{} Port:{}", config.host, config.port);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .expect("Invalid Host/Port specified");

    warp::serve(routes).run(addr).await;
}
