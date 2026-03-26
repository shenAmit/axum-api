use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client, RedisResult};

#[derive(Clone)]
pub struct Realtime {
    pub server_id: String,
    pub redis: ConnectionManager,
    pub redis_client: Client,
}

impl Realtime {
    pub async fn set_online(&self, user_id: &str) -> RedisResult<()> {
        let mut c = self.redis.clone();
        // Presence keys (use TTL so zombie connections eventually disappear)
        let _: () = c
            .set_ex(format!("presence:user:{}", user_id), &self.server_id, 90)
            .await?;
        let _: () = c.sadd("presence:online_users", user_id).await?;
        Ok(())
    }

    pub async fn set_offline(&self, user_id: &str) -> RedisResult<()> {
        let mut c = self.redis.clone();
        let _: () = c.del(format!("presence:user:{}", user_id)).await?;
        let _: () = c.srem("presence:online_users", user_id).await?;
        Ok(())
    }

    pub async fn join_room(&self, room: &str, user_id: &str) -> RedisResult<()> {
        let mut c = self.redis.clone();
        let _: () = c.sadd(format!("room:{}:members", room), user_id).await?;
        Ok(())
    }

    pub async fn leave_room(&self, room: &str, user_id: &str) -> RedisResult<()> {
        let mut c = self.redis.clone();
        let _: () = c.srem(format!("room:{}:members", room), user_id).await?;
        Ok(())
    }

    pub async fn publish_user(&self, user_id: &str, msg_json: &str) -> RedisResult<()> {
        let mut c = self.redis.clone();
        let _: () = c.publish(format!("chan:user:{}", user_id), msg_json).await?;
        Ok(())
    }

    pub async fn publish_room(&self, room: &str, msg_json: &str) -> RedisResult<()> {
        let mut c = self.redis.clone();
        let _: () = c.publish(format!("chan:room:{}", room), msg_json).await?;
        Ok(())
    }
}

