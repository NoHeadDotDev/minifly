use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    pub id: String,
    pub name: String,
    pub state: VolumeState,
    pub size_gb: u32,
    pub region: String,
    pub zone: String,
    pub encrypted: bool,
    pub attached_machine_id: Option<String>,
    pub attached_alloc_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VolumeState {
    Created,
    Creating,
    Updating,
    Destroying,
    Destroyed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVolumeRequest {
    pub name: String,
    pub region: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_gb: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fstype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_retention: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendVolumeRequest {
    pub size_gb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachVolumeRequest {
    pub machine_id: String,
}