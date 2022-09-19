use crate::assets;
use crate::cache;
use crate::Config;
use reqwest::Client;
use warp::{http::StatusCode, Rejection, Reply};
pub type PooledRedisConnection =
    r2d2_redis::r2d2::PooledConnection<r2d2_redis::RedisConnectionManager>;

pub async fn alive() -> Result<StatusCode, Rejection> {
    Ok(StatusCode::NO_CONTENT)
}

pub async fn ready() -> Result<StatusCode, Rejection> {
    Ok(StatusCode::OK)
}

pub async fn assetmap(
    config: Config,
    client: Client,
    redis_conn: PooledRedisConnection,
    addr: String,
    version: u64,
) -> Result<impl Reply, Rejection> {
    let cached_assetmap_res = cache::main_with_assetmap(addr.to_owned(), version, redis_conn);
    match cached_assetmap_res {
        Ok(cached_assetmap) => Ok(cached_assetmap),
        Err(_) => {
            let contents = assets::assetmap(&addr, version, client, &config).await?;
            Ok(contents)
        }
    }
}

pub async fn asset(
    config: Config,
    client: Client,
    redis_conn: PooledRedisConnection,
    addr: String,
    version: u64,
    file: String,
) -> Result<impl Reply, Rejection> {
    let cached_asset_res = cache::asset(addr.to_owned(), version, file.to_owned(), redis_conn);
    match cached_asset_res {
        Ok(cached_asset) => Ok(cached_asset),
        Err(_) => {
            let contents =
                assets::get_asset(&config, client.clone(), &addr, version, &file).await?;
            Ok(contents)
        }
    }
}
