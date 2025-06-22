use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

pub async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // For development, accept any Bearer token
    // In production, this would validate against actual tokens
    
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok());
    
    match auth_header {
        Some(auth) if auth.starts_with("Bearer ") => {
            // Token is present, continue
            Ok(next.run(request).await)
        }
        _ => {
            // No valid auth header
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}