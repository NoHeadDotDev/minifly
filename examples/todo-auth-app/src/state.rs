use axum::extract::FromRef;
use sqlx::{Pool, Sqlite};

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Sqlite>,
}

impl AppState {
    pub fn new(db: Pool<Sqlite>) -> Self {
        Self { db }
    }
}

impl FromRef<AppState> for Pool<Sqlite> {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}