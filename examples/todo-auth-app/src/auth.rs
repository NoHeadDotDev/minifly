use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    RequestPartsExt,
};
use tower_sessions::Session;

use crate::{
    error::{AppError, AppResult},
    models::{SessionUser, User},
    state::AppState,
};

const SESSION_USER_KEY: &str = "user";

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?;
    
    Ok(password_hash.to_string())
}

pub fn verify_password(password: &str, password_hash: &str) -> AppResult<bool> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|e| AppError::Internal(format!("Invalid password hash: {}", e)))?;
    
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

pub async fn set_session_user(session: &Session, user: &User) -> AppResult<()> {
    let session_user = SessionUser {
        id: user.id.clone(),
        email: user.email.clone(),
    };
    
    session
        .insert(SESSION_USER_KEY, session_user)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to set session: {}", e)))?;
    
    Ok(())
}

pub async fn get_session_user(session: &Session) -> AppResult<Option<SessionUser>> {
    session
        .get(SESSION_USER_KEY)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get session: {}", e)))
}

pub async fn clear_session(session: &Session) -> AppResult<()> {
    session.clear().await;
    Ok(())
}

// Extractor for authenticated users
pub struct AuthUser(pub SessionUser);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let session = parts
            .extract::<Session>()
            .await
            .map_err(|_| AppError::Internal("Failed to extract session".to_string()))?;

        let user = get_session_user(&session)
            .await?
            .ok_or(AppError::Unauthorized)?;

        Ok(AuthUser(user))
    }
}

// Optional auth extractor
pub struct OptionalAuthUser(pub Option<SessionUser>);

#[async_trait]
impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let session = parts
            .extract::<Session>()
            .await
            .map_err(|_| AppError::Internal("Failed to extract session".to_string()))?;

        let user = get_session_user(&session).await?;
        Ok(OptionalAuthUser(user))
    }
}