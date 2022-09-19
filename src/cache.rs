use r2d2_redis::redis;
use std::ops::DerefMut;

use crate::errors::AppError;
use warp::reject::custom as rejection;
use warp::Rejection;

pub type PooledRedisConnection =
    r2d2_redis::r2d2::PooledConnection<r2d2_redis::RedisConnectionManager>;

pub fn main_with_assetmap(
    address: String,
    version: u64,
    mut redis_conn: PooledRedisConnection,
) -> Result<String, Rejection> {
    let key = &format!("main::{}:{}", address, version);

    match redis::cmd("GET")
        .arg(key)
        .query::<String>(redis_conn.deref_mut())
    {
        Ok(reply) => {
            //Ok(serde_json::from_str(&reply).unwrap())
            Ok(reply)
        }
        Err(err) => {
            error!("Failed to find asset in cache: {}", err);
            Err(rejection(AppError::CacheMiss))
        }
    }
}

pub fn asset(
    address: String,
    version: u64,
    asset_file: String,
    mut redis_conn: PooledRedisConnection,
) -> Result<std::vec::Vec<u8>, Rejection> {
    let key = &format!("asset::{}:{}:{}", address, version, asset_file);

    match redis::cmd("GET")
        .arg(key)
        .query::<std::vec::Vec<u8>>(redis_conn.deref_mut())
    {
        Ok(reply) => {
            if reply.is_empty() {
                Err(rejection(AppError::CacheMiss))
            } else {
                Ok(reply)
            }
        }
        Err(err) => {
            error!("Failed to get value from cache: {}", err);
            Err(rejection(AppError::CacheMiss))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cache::*;
    use crate::config::*;
    use r2d2_redis::r2d2;
    use std::time;
    pub type RedisPool = r2d2_redis::r2d2::Pool<r2d2_redis::RedisConnectionManager>;
    use r2d2_redis::RedisConnectionManager;

    fn connection() -> RedisPool {
        let config = Config::new().expect("Failed to load config");
        info!("{:?}", config);

        let redis_conn_manager = RedisConnectionManager::new(config.redis_host).unwrap();

        let redis_pool = r2d2::Pool::builder()
            .max_size(10)
            .connection_timeout(time::Duration::from_millis(3000))
            .build_unchecked(redis_conn_manager);

        redis_pool
    }

    //Tests dependent on external services should be run explicitly
    #[tokio::test]
    #[ignore]
    async fn test_set_get_main_with_assetmap() {
        let data_val = "{\"hello\":\"world\",\"test\":\"case\"}";
        let mut conn = connection().get().unwrap();
        let key = &format!("main::{}:{}", String::from("123445"), 1);
        match redis::cmd("SET")
            .arg(key)
            .arg(data_val)
            .query::<String>(conn.deref_mut())
        {
            Ok(_) => {
                let value_res = main_with_assetmap(String::from("123445"), 1, conn);
                let value = format!("{}", value_res.unwrap());
                assert_eq!(value, data_val)
            }
            Err(err) => {
                error!("Failed to get value from cache: {}", err);
            }
        }
    }
}
