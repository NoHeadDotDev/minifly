/// Region context middleware for adding region information to responses and logs
/// 
/// This middleware:
/// - Adds region information to all API responses via headers
/// - Injects region context into the logging framework
/// - Tracks requests with correlation IDs for better debugging
use axum::{
    extract::Request,
    http::{HeaderMap, HeaderValue},
    middleware::Next,
    response::Response,
};
use tracing::{info, instrument, Span};
use uuid::Uuid;
use minifly_logging::fields;

/// Header name for region information
pub const REGION_HEADER: &str = "x-minifly-region";

/// Header name for correlation ID
pub const CORRELATION_ID_HEADER: &str = "x-minifly-correlation-id";

/// Default region for local development
pub const DEFAULT_REGION: &str = "local";

/// Middleware to add region context to requests and responses
/// 
/// This function:
/// 1. Generates a unique correlation ID for each request
/// 2. Adds region information to response headers
/// 3. Injects structured logging with region and correlation context
/// 4. Tracks request duration and outcomes
#[instrument(
    name = "region_middleware",
    skip_all,
    fields(
        region = %DEFAULT_REGION,
        correlation_id = tracing::field::Empty,
        request_id = tracing::field::Empty,
        http.method = %request.method(),
        http.path = %request.uri().path(),
        http.user_agent = tracing::field::Empty,
        http.status = tracing::field::Empty,
        duration_ms = tracing::field::Empty,
    )
)]
pub async fn region_middleware(request: Request, next: Next) -> Response {
    let correlation_id = minifly_logging::new_correlation_id();
    let request_id = minifly_logging::new_request_id();
    let region = DEFAULT_REGION.to_string();
    
    // Record structured fields in span
    Span::current().record(fields::CORRELATION_ID, &correlation_id);
    Span::current().record(fields::REQUEST_ID, &request_id);
    Span::current().record(fields::REGION, &region);
    
    // Extract user agent if present
    if let Some(user_agent) = request.headers().get("user-agent") {
        if let Ok(ua_str) = user_agent.to_str() {
            Span::current().record(fields::HTTP_USER_AGENT, ua_str);
        }
    }
    
    info!(
        operation = "http_request_start",
        "Processing HTTP request"
    );
    
    let start_time = std::time::Instant::now();
    
    // Process the request
    let mut response = next.run(request).await;
    
    let duration = start_time.elapsed();
    
    // Record final span fields
    Span::current().record(fields::HTTP_STATUS, response.status().as_u16());
    Span::current().record(fields::DURATION_MS, duration.as_millis());
    
    // Add region and correlation headers to response
    let headers = response.headers_mut();
    add_region_headers(headers, &region, &correlation_id);
    
    info!(
        operation = "http_request_complete",
        operation.status = "success",
        "HTTP request completed successfully"
    );
    
    response
}

/// Add region and correlation headers to the response
/// 
/// # Arguments
/// * `headers` - Response headers to modify
/// * `region` - Region identifier
/// * `correlation_id` - Request correlation ID
fn add_region_headers(headers: &mut HeaderMap, region: &str, correlation_id: &str) {
    if let Ok(region_value) = HeaderValue::from_str(region) {
        headers.insert(REGION_HEADER, region_value);
    }
    
    if let Ok(correlation_value) = HeaderValue::from_str(correlation_id) {
        headers.insert(CORRELATION_ID_HEADER, correlation_value);
    }
}

/// Extract region from machine information for logging context
/// 
/// # Arguments
/// * `machine_region` - Optional machine region, defaults to local
/// 
/// # Returns
/// * Region string for logging and response headers
pub fn get_machine_region(machine_region: Option<&str>) -> String {
    machine_region.unwrap_or(DEFAULT_REGION).to_string()
}

/// Log machine operation with region context
/// 
/// # Arguments
/// * `operation` - Operation being performed (e.g., "start", "stop", "create")
/// * `machine_id` - Machine identifier
/// * `app_name` - Application name
/// * `region` - Region where operation is occurring
#[instrument(skip_all)]
pub fn log_machine_operation(operation: &str, machine_id: &str, app_name: &str, region: &str) {
    info!(
        operation = %operation,
        machine_id = %machine_id,
        app_name = %app_name,
        region = %region,
        "Machine operation"
    );
}

/// Create structured log context for API operations
/// 
/// This macro helps create consistent logging across all API endpoints
/// with region and correlation information.
#[macro_export]
macro_rules! api_log {
    ($level:ident, $($field:ident = $value:expr),* $(,)? ; $($arg:tt)*) => {
        tracing::$level!(
            region = %crate::middleware::region::DEFAULT_REGION,
            $($field = $value,)*
            $($arg)*
        )
    };
}