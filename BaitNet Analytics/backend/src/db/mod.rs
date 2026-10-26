use sqlx::PgPool;

pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./src/db/migrations").run(pool).await
}
