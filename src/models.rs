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
    #[schema(example = "103956139596074306830")]
    pub author: String,
    #[schema(example = "2023-03-20")]
    pub upload_date: NaiveDate,
    #[schema(example = "SardineCanOpeningSimulatorTheGame")]
    pub name: String,
    #[schema(example = "kisQdebh0jnh6rb+bqQeM1EAxrg=")]
    pub hash: String,
    #[schema(example = "Huh")]
    pub description: String,
    #[schema(
        example = "[{\"name\": \"authrequired\", \"description\": \"Required CSH Authentication to Access\"}]"
    )]
    pub tags: Vec<Tag>,
    #[schema(
        example = "{
            \"id\": \"103956139596074306830\",
            \"user_type\": \"GOOGLE\",
            \"first_name\": \"Wilson\",
            \"last_name\": \"Mcdade\",
            \"picture\": \"https://lh3.googleusercontent.com/a/AGNmyxYS7ZmwC4Uw2ZhBlOdMvpIU7z3FwfRRkvw66d3r=s96-c\",
            \"admin\": false,
            \"email\": \"wam2134@g.rit.edu\"
        }"
    )]
    user: User,
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

#[derive(sqlx::Type, Serialize, Deserialize, Clone, ToSchema, Debug)]
pub enum UserType {
    CSH,
    GOOGLE,
}

#[derive(sqlx::Type, Serialize, Deserialize, FromRow, ToSchema)]
#[sqlx(type_name = "users")]
pub struct User {
    #[schema(example = "skyz")]
    pub id: String,
    #[schema(example = UserType::CSH)]
    pub user_type: UserType,
    #[schema(example = "Joe")]
    pub first_name: String,
    #[schema(example = "Abbate")]
    pub last_name: String,
    #[schema(example = "IMAGE_URL")]
    pub picture: String,
    #[schema(example = true)]
    pub admin: bool,
    #[schema(example = "skyz@csh.rit.edu")]
    pub email: String,
}

pub struct AppState {
    pub db: Pool<Postgres>,
    pub s3: Client,
}
