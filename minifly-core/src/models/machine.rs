use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Machine {
    pub id: String,
    pub name: String,
    pub state: MachineState,
    pub region: String,
    pub image_ref: ImageRef,
    pub instance_id: String,
    pub private_ip: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub config: MachineConfig,
    pub events: Vec<MachineEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MachineState {
    Created,
    Starting,
    Started,
    Stopping,
    Stopped,
    Destroying,
    Destroyed,
    Suspending,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRef {
    pub registry: String,
    pub repository: String,
    pub tag: String,
    pub digest: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineConfig {
    pub image: String,
    pub guest: GuestConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub services: Option<Vec<ServiceConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<HashMap<String, HealthCheck>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart: Option<RestartConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_destroy: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns: Option<DnsConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processes: Option<Vec<ProcessConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<FileConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub init: Option<InitConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mounts: Option<Vec<MountConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub containers: Option<Vec<ContainerConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestConfig {
    pub cpu_kind: String,
    pub cpus: u32,
    pub memory_mb: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpus: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernel_args: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub ports: Vec<PortConfig>,
    pub protocol: String,
    pub internal_port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autostop: Option<AutostopConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autostart: Option<AutostartConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_instance_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConfig {
    pub port: u16,
    pub handlers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_https: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_options: Option<TlsOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsOptions {
    pub alpn: Vec<String>,
    pub versions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    #[serde(rename = "type")]
    pub check_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grace_period: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_server_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_skip_verify: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartConfig {
    pub policy: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_registration: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nameservers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub searches: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exec: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secrets: Option<Vec<SecretConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretConfig {
    pub env_var: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    pub guest_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exec: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountConfig {
    pub volume: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    pub name: String,
    pub image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_checks: Option<Vec<HealthCheck>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub startup_commands: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attached_files: Option<Vec<FileConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutostopConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seconds: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutostartConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub status: String,
    pub source: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMachineRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    pub config: MachineConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_launch: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_service_registration: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lease_ttl: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMachineRequest {
    pub config: MachineConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_launch: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_service_registration: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lease_ttl: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopMachineRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartMachineResponse {
    pub previous_state: String,
    pub migrated: bool,
    pub new_host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopMachineResponse {
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitMachineQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}