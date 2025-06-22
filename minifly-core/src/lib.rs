pub mod error;
pub mod models;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

#[cfg(test)]
mod tests {
    use crate::models::{App, AppStatus, CreateAppRequest};
    use uuid::Uuid;
    
    #[test]
    fn test_app_creation() {
        let app = App {
            id: Uuid::new_v4(),
            name: "test-app".to_string(),
            organization_id: "test-org".to_string(),
            status: AppStatus::Deployed,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        assert_eq!(app.name, "test-app");
        assert_eq!(app.organization_id, "test-org");
        assert_eq!(app.status, AppStatus::Deployed);
    }
    
    #[test]
    fn test_app_status_serialization() {
        let status = AppStatus::Deployed;
        let serialized = serde_json::to_string(&status).unwrap();
        assert_eq!(serialized, "\"deployed\"");
        
        let deserialized: AppStatus = serde_json::from_str("\"suspended\"").unwrap();
        assert_eq!(deserialized, AppStatus::Suspended);
    }
    
    #[test]
    fn test_create_app_request() {
        let req = CreateAppRequest {
            app_name: "my-app".to_string(),
            org_slug: "my-org".to_string(),
        };
        
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"app_name\":\"my-app\""));
        assert!(json.contains("\"org_slug\":\"my-org\""));
    }
}