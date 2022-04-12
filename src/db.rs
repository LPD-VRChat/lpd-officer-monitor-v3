use dotenv::dotenv;
use entity::sea_orm::ConnectOptions;
use entity::sea_orm::Database;
use entity::sea_orm::DatabaseConnection;
use std::env;
use std::time::Duration;

pub async fn establish_connection() -> DatabaseConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let mut opt = ConnectOptions::new(database_url.to_owned());
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true);

    Database::connect(opt)
        .await
        .expect(&format!("Error connecting to {}", database_url))
}
