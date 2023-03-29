use aws_sdk_s3::Client;
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::{PgHasArrayType, PgTypeInfo},
    types::chrono::NaiveDate,
    FromRow, Pool, Postgres,
};
use utoipa::{self, ToSchema};

#[derive(Serialize, Deserialize, FromRow, ToSchema)]
pub struct Game {
    #[schema(example = "a1c6cef6-d987-4225-8bc4-def387e8b5bf")]
    pub id: String,
    #[schema(example = "ella")]
    pub author: String,
    #[schema(example = "2023-03-20")]
    pub upload_date: NaiveDate,
    #[schema(example = "BrickBreaker")]
    pub name: String,
    #[schema(example = "kisQdebh0jnh6rb+bqQeM1EAxrg=")]
    pub hash: String,
    #[schema(example = "Break bricks, get points")]
    pub description: String,
}

#[derive(Serialize, Deserialize, FromRow, ToSchema)]
pub struct GameWithTags {
    #[schema(example = "a1c6cef6-d987-4225-8bc4-def387e8b5bf")]
    pub id: String,
    #[schema(example = "ella")]
    pub author: String,
    #[schema(example = "2023-03-20")]
    pub upload_date: NaiveDate,
    #[schema(example = "BrickBreaker")]
    pub name: String,
    #[schema(example = "kisQdebh0jnh6rb+bqQeM1EAxrg=")]
    pub hash: String,
    #[schema(example = "Break bricks, get points")]
    pub description: String,
    #[schema(
        example = "[{\"name\": \"authrequired\", \"description\": \"Required CSH Authentication to Access\"}]"
    )]
    pub tags: Vec<Tag>,
}

#[derive(sqlx::Type, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Tag {
    #[schema(example = "authrequired")]
    pub name: String,
    #[schema(example = "Required CSH Authentication to Access")]
    pub description: String,
}

impl PgHasArrayType for Tag {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_tags")
    }
}

pub struct AppState {
    pub db: Pool<Postgres>,
    pub s3: Client,
}
