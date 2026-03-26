use redis::aio::ConnectionManager;
use redis::Client;

pub async fn connect_redis(redis_url: &str) -> redis::RedisResult<ConnectionManager> {
    let client = Client::open(redis_url)?;
    let manager = client.get_connection_manager().await?;
    Ok(manager)
}

