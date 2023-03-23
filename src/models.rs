use aws_sdk_s3::Client;
use serde::{Deserialize, Serialize};
use sqlx::{types::chrono::NaiveDate, FromRow, Pool, Postgres};
use utoipa::{self, ToSchema};

#[derive(Serialize, Deserialize, FromRow, ToSchema)]
pub struct Game {
    #[schema(example = "a1c6cef6-d987-4225-8bc4-def387e8b5bf")]
    pub game_id: String,
    #[schema(example = "ella")]
    pub author_username: String,
    #[schema(example = "2023-03-20")]
    pub upload_date: NaiveDate,
    #[schema(example = "BrickBreaker")]
    pub game_name: String,
    #[schema(example = "kisQdebh0jnh6rb+bqQeM1EAxrg=")]
    pub hash: String,
    #[schema(example = "Break bricks, get points")]
    pub description: String,
    #[schema(example = false)]
    pub authrequired: bool,
}

pub struct AppState {
    pub db: Pool<Postgres>,
    pub s3: Client,
}
