use scrobblify_db::migrator;
use scrobblify_db::migrator::sea_orm_migration::prelude::*;

// TODO: set/check DATABASE_URL env

#[tokio::main]
async fn main() {
    cli::run_cli(migrator::Migrator).await;
}
