use actix_web::{
    error,
    Result,
};
use async_trait::async_trait;
use redis::aio::ConnectionManager;

use crate::support::id::Identifier;
use crate::{
    app::config::Db,
    support::{
        id::ID,
        page::Page,
    },
};

use super::entity::Driver;

#[async_trait]
pub(crate) trait Repository {
    async fn exists(&mut self, key: &str) -> Result<bool>;

    async fn list(&mut self, page: &Page) -> Result<Vec<ID<Driver>>>;

    async fn count(&mut self) -> Result<isize>;

    async fn add(&mut self, drv: &ID<Driver>) -> Result<()>;
}

struct RedisRepository {
    conn: ConnectionManager,
}

#[async_trait]
impl Repository for RedisRepository {
    async fn exists(&mut self, key: &str) -> Result<bool> {
        redis::Cmd::exists(key)
            .query_async(&mut self.conn)
            .await
            .map_err(error::ErrorInternalServerError)
    }

    async fn list(&mut self, page: &Page) -> Result<Vec<ID<Driver>>> {
        let query =
            redis::Cmd::zrange("drivers-idx", page.start(), page.stop());

        let ids: Vec<String> = query
            .query_async(&mut self.conn)
            .await
            .map_err(error::ErrorInternalServerError)?;

        let keys: Vec<String> =
            ids.iter().map(|id| format!("drivers:{}", id)).collect();

        log::debug!("keys: {:?}", keys);

        if keys.is_empty() {
            return Ok(vec![]);
        }

        let query = redis::Cmd::json_get(keys, "$")
            .map_err(error::ErrorInternalServerError)?;

        let drvs: Vec<String> = query
            .query_async(&mut self.conn)
            .await
            .map_err(error::ErrorInternalServerError)?;

        log::trace!("drvs: {:?}", drvs);

        let drvs: Result<Vec<Vec<Driver>>> = drvs
            .into_iter()
            .map(|drv| {
                serde_json::from_str(&drv)
                    .map_err(error::ErrorInternalServerError)
            })
            .collect();

        let drvs = drvs?;

        let drvs: Vec<Driver> = drvs.into_iter().flatten().collect();

        log::trace!("drvs: {:?}", drvs);

        let drvs: Vec<ID<Driver>> = drvs
            .iter()
            .zip(ids.iter())
            .map(|(drv, id)| ID {
                id:     Identifier::from(id),
                entity: drv.clone(),
            })
            .collect();

        log::debug!("drvs: {:?}", drvs);
        Ok(drvs)
    }

    async fn count(&mut self) -> Result<isize> {
        redis::Cmd::zcard("drivers-idx")
            .query_async(&mut self.conn)
            .await
            .map_err(error::ErrorInternalServerError)
    }

    async fn add(&mut self, drv: &ID<Driver>) -> Result<()> {
        let id = drv.id.to_string();
        let key = format!("drivers:{}", &id);
        let query = redis::Cmd::json_set(key, "$", &drv.entity)
            .map_err(error::ErrorInternalServerError)?;
        query
            .query_async(&mut self.conn)
            .await
            .map_err(error::ErrorInternalServerError)?;

        redis::Cmd::zadd("drivers-idx", &id, drv.id.int())
            .query_async(&mut self.conn)
            .await
            .map_err(error::ErrorInternalServerError)?;

        Ok(())
    }
}

pub(crate) async fn new(db: Db) -> Result<Box<dyn Repository>> {
    let client = db
        .client
        .ok_or(error::ErrorInternalServerError("No redis client"))?;
    let conn = client
        .get_tokio_connection_manager()
        .await
        .map_err(error::ErrorInternalServerError)?;
    Ok(Box::new(RedisRepository { conn }))
}
