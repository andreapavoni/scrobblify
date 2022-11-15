use std::env;

use scrobblify_db::migrator;
use scrobblify_db::migrator::sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    if env::var_os("DATABASE_URL") == None {
        panic!("$DATABASE_URL is not set")
    }
    cli::run_cli(migrator::Migrator).await;
}
