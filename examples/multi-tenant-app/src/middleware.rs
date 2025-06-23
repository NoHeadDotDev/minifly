use axum::{
    http::{HeaderMap, Request},
    middleware::Next,
    response::Response,
};
use tracing::debug;

#[derive(Clone)]
pub struct TenantId(pub String);

pub async fn extract_tenant(
    mut request: Request<axum::body::Body>,
    next: Next<axum::body::Body>,
) -> Response {
    // Extract tenant from various sources
    let headers = request.headers();
    let tenant = extract_tenant_id(headers, &request);
    
    debug!("Processing request for tenant: {}", tenant);
    
    // Insert tenant ID into request extensions
    request.extensions_mut().insert(TenantId(tenant));
    
    // Continue processing
    next.run(request).await
}

fn extract_tenant_id(headers: &HeaderMap, request: &Request<axum::body::Body>) -> String {
    // 1. Check X-Tenant header
    if let Some(tenant_header) = headers.get("X-Tenant") {
        if let Ok(tenant) = tenant_header.to_str() {
            return sanitize_tenant_id(tenant);
        }
    }
    
    // 2. Check Host header for subdomain
    if let Some(host_header) = headers.get("Host") {
        if let Ok(host) = host_header.to_str() {
            if let Some(tenant) = extract_subdomain_tenant(host) {
                return tenant;
            }
        }
    }
    
    // 3. Check URL path
    let path = request.uri().path();
    if let Some(tenant) = extract_path_tenant(path) {
        return tenant;
    }
    
    // Default tenant
    "default".to_string()
}

fn extract_subdomain_tenant(host: &str) -> Option<String> {
    // Extract tenant from subdomain (e.g., "tenant1.example.com")
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() > 2 {
        let tenant = parts[0];
        if tenant != "www" && !tenant.is_empty() {
            return Some(sanitize_tenant_id(tenant));
        }
    }
    None
}

fn extract_path_tenant(path: &str) -> Option<String> {
    // Extract tenant from path (e.g., "/tenant/tenant1/...")
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() >= 2 && parts[0] == "tenant" {
        return Some(sanitize_tenant_id(parts[1]));
    }
    None
}

fn sanitize_tenant_id(tenant: &str) -> String {
    // Sanitize tenant ID to prevent directory traversal and invalid characters
    tenant
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .take(50) // Limit length
        .collect::<String>()
        .to_lowercase()
}