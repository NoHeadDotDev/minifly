use axum::response::Html;
use askama::Template;

use crate::{
    auth::OptionalAuthUser,
    error::AppResult,
    templates::HomePage,
};

pub async fn index(
    OptionalAuthUser(user): OptionalAuthUser,
) -> AppResult<Html<String>> {
    let template = HomePage { user };
    Ok(Html(template.render().unwrap()))
}