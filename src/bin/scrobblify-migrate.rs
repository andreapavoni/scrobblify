use scrobblify::db::migrator;
use scrobblify::db::migrator::sea_orm_migration::prelude::*;
use std::env;

#[tokio::main]
async fn main() {
    if env::var_os("DATABASE_URL") == None {
        panic!("$DATABASE_URL is not set")
    }
    cli::run_cli(migrator::Migrator).await;
}
