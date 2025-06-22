use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use tracing::{debug, info, warn};

/// DNS resolver for .internal domains in Minifly.
/// 
/// This resolver simulates Fly.io's internal DNS resolution for local development.
/// It maintains mappings between app names, machine IDs, and their IP addresses,
/// allowing applications to resolve `.internal` domains just like they would in production.
/// 
/// # Supported Domain Formats
/// 
/// - `<app>.internal` - Resolves to all machine IPs for the app
/// - `<machine-id>.vm.<app>.internal` - Resolves to specific machine IP
/// - `fly-local-6pn.internal` - Special domain for local Docker DNS
/// 
/// # Example
/// 
/// ```rust
/// use minifly_network::InternalDnsResolver;
/// use std::net::{IpAddr, Ipv4Addr};
/// 
/// # tokio_test::block_on(async {
/// let resolver = InternalDnsResolver::new();
/// let ip = IpAddr::V4(Ipv4Addr::new(172, 19, 0, 2));
/// 
/// // Register a machine
/// resolver.register_machine("myapp", "machine-1", ip).await.unwrap();
/// 
/// // Resolve app.internal
/// let ips = resolver.resolve("myapp.internal").await.unwrap();
/// assert_eq!(ips, vec![ip]);
/// 
/// // Resolve machine-specific domain
/// let ips = resolver.resolve("machine-1.vm.myapp.internal").await.unwrap();
/// assert_eq!(ips, vec![ip]);
/// # });
/// ```
pub struct InternalDnsResolver {
    /// Map of app names to their machine IPs
    app_ips: Arc<RwLock<HashMap<String, Vec<IpAddr>>>>,
    /// Map of machine IDs to their IPs
    machine_ips: Arc<RwLock<HashMap<String, IpAddr>>>,
}

impl InternalDnsResolver {
    /// Creates a new DNS resolver instance.
    /// 
    /// The resolver starts empty and machines must be registered
    /// before they can be resolved.
    pub fn new() -> Self {
        Self {
            app_ips: Arc::new(RwLock::new(HashMap::new())),
            machine_ips: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers a machine with its IP address for DNS resolution.
    /// 
    /// This allows the machine to be resolved via both `<app>.internal` and
    /// `<machine-id>.vm.<app>.internal` domains.
    /// 
    /// # Arguments
    /// 
    /// * `app_name` - The name of the application
    /// * `machine_id` - Unique identifier for the machine
    /// * `ip` - IP address of the machine
    /// 
    /// # Example
    /// 
    /// ```rust
    /// # use minifly_network::InternalDnsResolver;
    /// # use std::net::{IpAddr, Ipv4Addr};
    /// # tokio_test::block_on(async {
    /// let resolver = InternalDnsResolver::new();
    /// let ip = IpAddr::V4(Ipv4Addr::new(172, 19, 0, 2));
    /// 
    /// resolver.register_machine("myapp", "machine-1", ip).await.unwrap();
    /// # });
    /// ```
    pub async fn register_machine(&self, app_name: &str, machine_id: &str, ip: IpAddr) -> Result<()> {
        info!("Registering machine {} for app {} with IP {}", machine_id, app_name, ip);
        
        // Update machine IPs
        {
            let mut machine_ips = self.machine_ips.write().await;
            machine_ips.insert(machine_id.to_string(), ip);
        }
        
        // Update app IPs
        {
            let mut app_ips = self.app_ips.write().await;
            let ips = app_ips.entry(app_name.to_string()).or_insert_with(Vec::new);
            if !ips.contains(&ip) {
                ips.push(ip);
            }
        }
        
        Ok(())
    }

    /// Unregisters a machine from DNS resolution.
    /// 
    /// Removes the machine from both app-level and machine-specific DNS mappings.
    /// If this was the last machine for an app, the app domain will no longer resolve.
    /// 
    /// # Arguments
    /// 
    /// * `app_name` - The name of the application
    /// * `machine_id` - Unique identifier for the machine to remove
    pub async fn unregister_machine(&self, app_name: &str, machine_id: &str) -> Result<()> {
        info!("Unregistering machine {} for app {}", machine_id, app_name);
        
        // Remove from machine IPs
        let ip = {
            let mut machine_ips = self.machine_ips.write().await;
            machine_ips.remove(machine_id)
        };
        
        // Remove from app IPs if we found the IP
        if let Some(ip) = ip {
            let mut app_ips = self.app_ips.write().await;
            if let Some(ips) = app_ips.get_mut(app_name) {
                ips.retain(|&existing_ip| existing_ip != ip);
                if ips.is_empty() {
                    app_ips.remove(app_name);
                }
            }
        }
        
        Ok(())
    }

    /// Resolves a .internal domain to IP addresses.
    /// 
    /// Supports multiple domain formats for compatibility with Fly.io:
    /// - `<app>.internal` - Returns all machine IPs for the app
    /// - `<machine-id>.vm.<app>.internal` - Returns IP for specific machine
    /// - `fly-local-6pn.internal` - Returns local Docker DNS server IP
    /// 
    /// # Arguments
    /// 
    /// * `hostname` - The .internal domain to resolve
    /// 
    /// # Returns
    /// 
    /// A vector of IP addresses that the hostname resolves to.
    /// Returns an empty vector if the hostname cannot be resolved.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// # use minifly_network::InternalDnsResolver;
    /// # use std::net::{IpAddr, Ipv4Addr};
    /// # tokio_test::block_on(async {
    /// let resolver = InternalDnsResolver::new();
    /// let ip = IpAddr::V4(Ipv4Addr::new(172, 19, 0, 2));
    /// 
    /// resolver.register_machine("myapp", "machine-1", ip).await.unwrap();
    /// 
    /// // Resolve app domain
    /// let ips = resolver.resolve("myapp.internal").await.unwrap();
    /// assert_eq!(ips, vec![ip]);
    /// 
    /// // Resolve machine-specific domain
    /// let ips = resolver.resolve("machine-1.vm.myapp.internal").await.unwrap();
    /// assert_eq!(ips, vec![ip]);
    /// # });
    /// ```
    pub async fn resolve(&self, hostname: &str) -> Result<Vec<IpAddr>> {
        debug!("Resolving hostname: {}", hostname);
        
        // Handle special Fly.io domains
        if hostname == "fly-local-6pn.internal" {
            // Return the local Docker DNS server
            return Ok(vec![IpAddr::V4(Ipv4Addr::new(172, 17, 0, 1))]);
        }
        
        // Handle app.internal format
        if let Some(app_name) = hostname.strip_suffix(".internal") {
            let app_ips = self.app_ips.read().await;
            if let Some(ips) = app_ips.get(app_name) {
                debug!("Resolved {} to {:?}", hostname, ips);
                return Ok(ips.clone());
            }
        }
        
        // Handle machine-id.vm.app.internal format
        if hostname.ends_with(".internal") {
            let parts: Vec<&str> = hostname.split('.').collect();
            if parts.len() >= 4 && parts[parts.len() - 3] == "vm" {
                let machine_id = parts[0];
                let machine_ips = self.machine_ips.read().await;
                if let Some(ip) = machine_ips.get(machine_id) {
                    debug!("Resolved machine {} to {}", machine_id, ip);
                    return Ok(vec![*ip]);
                }
            }
        }
        
        warn!("Failed to resolve hostname: {}", hostname);
        Ok(vec![])
    }

    /// Returns all registered apps and their IP addresses.
    /// 
    /// This is primarily useful for debugging and monitoring the current
    /// state of registered machines.
    /// 
    /// # Returns
    /// 
    /// A HashMap mapping app names to vectors of their machine IP addresses.
    pub async fn list_registrations(&self) -> HashMap<String, Vec<IpAddr>> {
        self.app_ips.read().await.clone()
    }
}

/// Extracts a container's IP address from Docker container network information.
/// 
/// This helper function parses Docker's network settings JSON to find the first
/// available IP address for a container. It's used internally by Minifly to
/// automatically register container IPs with the DNS resolver.
/// 
/// # Arguments
/// 
/// * `networks` - JSON value containing Docker container network settings
/// 
/// # Returns
/// 
/// The first valid IP address found, or `None` if no valid IP is available.
/// 
/// # Example
/// 
/// ```rust
/// use minifly_network::extract_container_ip;
/// use serde_json::json;
/// 
/// let networks = json!({
///     "bridge": {
///         "IPAddress": "172.17.0.2"
///     }
/// });
/// 
/// let ip = extract_container_ip(&networks);
/// assert!(ip.is_some());
/// ```
pub fn extract_container_ip(networks: &serde_json::Value) -> Option<IpAddr> {
    // Try to find the first network with an IP address
    if let Some(networks_obj) = networks.as_object() {
        for (_name, network) in networks_obj {
            if let Some(ip_address) = network.get("IPAddress").and_then(|v| v.as_str()) {
                if !ip_address.is_empty() {
                    if let Ok(ip) = ip_address.parse::<IpAddr>() {
                        return Some(ip);
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dns_registration() {
        let resolver = InternalDnsResolver::new();
        
        // Register a machine
        let ip = IpAddr::V4(Ipv4Addr::new(172, 19, 0, 2));
        resolver.register_machine("myapp", "machine-1", ip).await.unwrap();
        
        // Resolve app.internal
        let ips = resolver.resolve("myapp.internal").await.unwrap();
        assert_eq!(ips.len(), 1);
        assert_eq!(ips[0], ip);
        
        // Resolve machine-id.vm.app.internal
        let ips = resolver.resolve("machine-1.vm.myapp.internal").await.unwrap();
        assert_eq!(ips.len(), 1);
        assert_eq!(ips[0], ip);
    }

    #[tokio::test]
    async fn test_multiple_machines() {
        let resolver = InternalDnsResolver::new();
        
        // Register multiple machines for the same app
        let ip1 = IpAddr::V4(Ipv4Addr::new(172, 19, 0, 2));
        let ip2 = IpAddr::V4(Ipv4Addr::new(172, 19, 0, 3));
        
        resolver.register_machine("myapp", "machine-1", ip1).await.unwrap();
        resolver.register_machine("myapp", "machine-2", ip2).await.unwrap();
        
        // Resolve app.internal should return both IPs
        let ips = resolver.resolve("myapp.internal").await.unwrap();
        assert_eq!(ips.len(), 2);
        assert!(ips.contains(&ip1));
        assert!(ips.contains(&ip2));
    }

    #[tokio::test]
    async fn test_unregister_machine() {
        let resolver = InternalDnsResolver::new();
        
        // Register and then unregister
        let ip = IpAddr::V4(Ipv4Addr::new(172, 19, 0, 2));
        resolver.register_machine("myapp", "machine-1", ip).await.unwrap();
        resolver.unregister_machine("myapp", "machine-1").await.unwrap();
        
        // Should not resolve anymore
        let ips = resolver.resolve("myapp.internal").await.unwrap();
        assert_eq!(ips.len(), 0);
    }
}