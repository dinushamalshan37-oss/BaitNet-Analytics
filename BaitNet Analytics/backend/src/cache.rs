use redis::{aio::ConnectionManager, AsyncCommands, Client};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Clone)]
pub struct Cache {
    connection: ConnectionManager,
    ttl_seconds: u64,
}

impl Cache {
    pub async fn connect(url: &str, ttl_seconds: u64) -> redis::RedisResult<Self> {
        let client = Client::open(url)?;
        let connection = ConnectionManager::new(client).await?;
        Ok(Self { connection, ttl_seconds })
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> redis::RedisResult<Option<T>> {
        let mut connection = self.connection.clone();
        let value: Option<String> = connection.get(key).await?;
        value
            .map(|json| serde_json::from_str(&json).map_err(|error| redis::RedisError::from((redis::ErrorKind::TypeError, "invalid cached JSON", error.to_string()))))
            .transpose()
    }

    pub async fn set<T: Serialize>(&self, key: &str, value: &T) -> redis::RedisResult<()> {
        let mut connection = self.connection.clone();
        let json = serde_json::to_string(value).map_err(|error| redis::RedisError::from((redis::ErrorKind::TypeError, "could not serialize cache value", error.to_string())))?;
        connection.set_ex(key, json, self.ttl_seconds).await
    }

    pub async fn allow_requests(&self, key: &str, max_requests: u64, window_seconds: u64) -> redis::RedisResult<bool> {
        let mut connection = self.connection.clone();
        let script = redis::Script::new(
            "local count = redis.call('INCR', KEYS[1]); if count == 1 then redis.call('EXPIRE', KEYS[1], ARGV[1]); end; return count",
        );
        let count: u64 = script.key(key).arg(window_seconds).invoke_async(&mut connection).await?;
        Ok(count <= max_requests)
    }

    pub async fn ping(&self) -> redis::RedisResult<()> {
        let mut connection = self.connection.clone();
        let _: String = redis::cmd("PING").query_async(&mut connection).await?;
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> redis::RedisResult<()> {
        let mut connection = self.connection.clone();
        let _: usize = connection.del(key).await?;
        Ok(())
    }
}
