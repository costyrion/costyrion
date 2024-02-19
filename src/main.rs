#![allow(dead_code, unused)]
use async_trait::async_trait;
use sqlx::postgres::PgPool;
use std::sync::Arc;

#[derive(Debug)]
struct Resource {
    id: Option<i32>,
    reference: Option<String>,
}

impl Resource {
    fn new(reference: String) -> Resource {
        Resource {
            id: None,
            reference: Some(reference),
        }
    }
}

#[async_trait]
trait ResourceRepository {
    async fn create_resource(&self, resource: String) -> anyhow::Result<i32>;
    async fn read_resource(&self, id: i32) -> anyhow::Result<Resource>;
    async fn update_resource(
        &self,
        id: i32,
        reference: String,
    ) -> anyhow::Result<bool>;
    async fn delete_resource(&self, id: i32) -> anyhow::Result<bool>;
}

struct PostgresResourceRepository {
    pg_pool: Arc<PgPool>,
}

impl PostgresResourceRepository {
    fn new(pg_pool: PgPool) -> Self {
        Self {
            pg_pool: Arc::new(pg_pool),
        }
    }
}

#[async_trait]
impl ResourceRepository for PostgresResourceRepository {
    async fn create_resource(&self, reference: String) -> anyhow::Result<i32> {
        let record = sqlx::query!(
            r#"
                INSERT INTO resources (reference)
                VALUES ( $1 )
                RETURNING id
            "#,
            reference
        )
        .fetch_one(&*self.pg_pool)
        .await?;

        Ok(record.id)
    }

    async fn read_resource(&self, id: i32) -> anyhow::Result<Resource> {
        let result = sqlx::query_as!(
            Resource,
            r#"SELECT id, reference FROM resources WHERE id = $1"#,
            id
        )
        .fetch_one(&*self.pg_pool)
        .await;

        match result {
            Ok(resource) => Ok(resource),
            Err(err) => {
                Err(anyhow::anyhow!("resource id: {} msg: {}", id, err))
            }
        }
    }

    async fn update_resource(
        &self,
        id: i32,
        reference: String,
    ) -> anyhow::Result<bool> {
        let rows_affected = sqlx::query!(
            r#"UPDATE resources SET reference = $1 WHERE id = $2"#,
            reference,
            id
        )
        .execute(&*self.pg_pool)
        .await?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    async fn delete_resource(&self, id: i32) -> anyhow::Result<bool> {
        let rows_affected =
            sqlx::query!(r#"DELETE FROM resources WHERE id = $1"#, id)
                .execute(&*self.pg_pool)
                .await?
                .rows_affected();

        Ok(rows_affected > 0)
    }
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // load variables from .env
    dotenvy::dotenv().expect("Failed to load .env file");

    let db_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL not defined");

    let pool = PgPool::connect(&db_url).await?;
    let repo = PostgresResourceRepository::new(pool);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_create_and_read() {
        let pg_pool =
            PgPool::connect("postgresql://postgres:postgres@localhost:5432")
                .await
                .expect("Unable to connect to DB");

        sqlx::query("DROP TABLE IF EXISTS resources")
            .execute(&pg_pool)
            .await
            .unwrap();

        sqlx::query(
            "CREATE TABLE resources (id SERIAL PRIMARY KEY, reference VARCHAR(255))",
        )
        .execute(&pg_pool)
        .await
        .unwrap();

        let repo = PostgresResourceRepository {
            pg_pool: Arc::new(pg_pool),
        };

        assert_eq!(1, repo.create_resource("TEST".to_string()).await.unwrap());
    }
}
