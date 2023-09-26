pub mod entities;

use anyhow::{Context, Result};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sqlx::postgres::PgPoolOptions;
// use sqlx::{migrate, postgres::PgPoolOptions};

use crate::ssr::config_this::ConfigThis;

pub type PoolDatabasePostgresConnection = DatabaseConnection;

async fn do_migrations(_db: &sqlx::Pool<sqlx::Postgres>) -> Result<()> {
  // migrate!("src/features/databases/postgres/migrations")
  //   .run(db)
  //   .await
  //   .context("postgeSQL migration")

  Ok(())
}

pub async fn get_connection_pool(
  config: &ConfigThis,
) -> Result<(sqlx::Pool<sqlx::Postgres>, DatabaseConnection)> {
  // https://github.com/launchbadge/sqlx/issues/916
  let db: sqlx::Pool<sqlx::Postgres> = PgPoolOptions::new()
    .max_connections(20)
    .acquire_timeout(std::time::Duration::from_secs(3))
    .connect_lazy(&config.databases.postgres.uri)
    .context("connect to PostgreSQL")?;

  if let Err(err) = do_migrations(&db).await {
    tracing::warn!("{}", err.to_string());

    sqlx::query("DROP TABLE IF EXISTS _sqlx_migrations;")
      .execute(&db)
      .await?;

    do_migrations(&db).await?;
  };

  let mut options_connect = ConnectOptions::new::<&str>(config.databases.postgres.uri.as_ref());

  options_connect
    .max_connections(20)
    .acquire_timeout(std::time::Duration::from_secs(3))
    .connect_timeout(std::time::Duration::from_secs(3))
    .sqlx_logging(true)
    .sqlx_logging_level(log::LevelFilter::Info);

  let db_connection = Database::connect(options_connect)
    .await
    .context("connect to PosstgreSQL")?;

  tracing::info!("PostgreSQL connected.");

  Ok((db, db_connection))
}
