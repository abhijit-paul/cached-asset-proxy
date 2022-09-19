use crate::handlers;

use crate::Config;

use reqwest::Client;
use std::convert::Infallible;
use warp::Filter;

pub type RedisPool = r2d2_redis::r2d2::Pool<r2d2_redis::RedisConnectionManager>;
pub type PooledRedisConnection =
    r2d2_redis::r2d2::PooledConnection<r2d2_redis::RedisConnectionManager>;

fn with_client(client: Client) -> impl Filter<Extract = (Client,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

fn with_config(config: Config) -> impl Filter<Extract = (Config,), Error = Infallible> + Clone {
    warp::any().map(move || config.clone())
}

fn with_redis_conn(
    redis_pool: RedisPool,
) -> impl Filter<Extract = (PooledRedisConnection,), Error = Infallible> + Clone {
    warp::any().map(move || redis_pool.get().unwrap())
}

pub fn alive() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("alive"))
        .and(warp::path::end())
        .and_then(handlers::alive)
}

pub fn ready() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("ready"))
        .and(warp::path::end())
        .and_then(handlers::ready)
}

pub fn assetmap(
    config: Config,
    client: Client,
    redis_pool: RedisPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(with_config(config))
        .and(with_client(client))
        .and(with_redis_conn(redis_pool))
        .and(warp::path!("api" / "assetmap" / String / u64))
        .and(warp::path::end())
        .and_then(handlers::assetmap)
}

pub fn asset(
    config: Config,
    client: Client,
    redis_pool: RedisPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(with_config(config))
        .and(with_client(client))
        .and(with_redis_conn(redis_pool))
        .and(warp::path!("api" / String / u64 / "asset" / String))
        .and(warp::path::end())
        .and_then(handlers::asset)
}
