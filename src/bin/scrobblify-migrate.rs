use scrobblify::db::migrator;
use scrobblify::db::migrator::sea_orm_migration::prelude::*;
use std::env;

#[tokio::main]
async fn main() {
    match env::var_os("DATABSE_URL") {
        None => panic!("$DATABSE_URL is not set"),
        _ => {}
    }
    cli::run_cli(migrator::Migrator).await;
}
