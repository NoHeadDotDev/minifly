#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, RwLock};
    use std::collections::HashMap;
    use uuid::Uuid;
    
    fn test_state() -> AppState {
        AppState {
            config: Arc::new(Config {
                port: 8080,
                docker_host: None,
                data_dir: "/tmp/test".into(),
                log_level: "debug".to_string(),
                litefs_config_dir: "/tmp/test/litefs".into(),
            }),
            apps: Arc::new(RwLock::new(HashMap::new())),
            machines: Arc::new(RwLock::new(HashMap::new())),
            volumes: Arc::new(RwLock::new(HashMap::new())),
            leases: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(Vec::new())),
            docker: None,
        }
    }
    
    #[test]
    fn test_app_storage() {
        let state = test_state();
        
        // Create test app
        let app = App {
            id: Uuid::new_v4().to_string(),
            name: "test-app".to_string(),
            organization: Organization {
                id: "test-org".to_string(),
                slug: "test-org".to_string(),
                name: "Test Organization".to_string(),
            },
            status: AppStatus::Running,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        // Store app
        {
            let mut apps = state.apps.write().unwrap();
            apps.insert(app.name.clone(), app.clone());
        }
        
        // Retrieve app
        {
            let apps = state.apps.read().unwrap();
            let stored_app = apps.get("test-app").unwrap();
            assert_eq!(stored_app.name, "test-app");
            assert_eq!(stored_app.organization.slug, "test-org");
        }
        
        // Delete app
        {
            let mut apps = state.apps.write().unwrap();
            apps.remove("test-app");
        }
        
        // Verify deletion
        {
            let apps = state.apps.read().unwrap();
            assert!(apps.get("test-app").is_none());
        }
    }
    
    #[test]
    fn test_machine_storage() {
        let state = test_state();
        
        // Create test machine
        let machine = Machine {
            id: format!("d{}", Uuid::new_v4().to_string().replace("-", "")[..15]),
            name: "test-machine".to_string(),
            state: MachineState::Created,
            region: "local".to_string(),
            config: MachineConfig {
                image: "nginx:latest".to_string(),
                guest: Some(GuestConfig {
                    cpus: 1,
                    memory_mb: 256,
                    cpu_kind: CpuKind::Shared,
                }),
                env: HashMap::new(),
                services: vec![],
                mounts: vec![],
                size: MachineSize::Shared1x,
                restart: None,
                auto_destroy: false,
                schedule: None,
            },
            app_name: "test-app".to_string(),
            private_ip: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version: 1,
            events: vec![],
            checks: HashMap::new(),
            nonce: None,
        };
        
        // Store machine
        {
            let mut machines = state.machines.write().unwrap();
            machines.insert(machine.id.clone(), machine.clone());
        }
        
        // Retrieve machine
        {
            let machines = state.machines.read().unwrap();
            let stored_machine = machines.get(&machine.id).unwrap();
            assert_eq!(stored_machine.name, "test-machine");
            assert_eq!(stored_machine.app_name, "test-app");
        }
        
        // Update machine state
        {
            let mut machines = state.machines.write().unwrap();
            if let Some(m) = machines.get_mut(&machine.id) {
                m.state = MachineState::Started;
                m.updated_at = chrono::Utc::now();
            }
        }
        
        // Verify update
        {
            let machines = state.machines.read().unwrap();
            let updated_machine = machines.get(&machine.id).unwrap();
            assert_eq!(updated_machine.state, MachineState::Started);
        }
    }
    
    #[test]
    fn test_volume_storage() {
        let state = test_state();
        
        // Create test volume
        let volume = Volume {
            id: format!("vol_{}", Uuid::new_v4().to_string().replace("-", "")[..10]),
            name: "test-volume".to_string(),
            app_id: "test-app".to_string(),
            size_gb: 10,
            region: "local".to_string(),
            zone: "local-a".to_string(),
            encrypted: true,
            created_at: chrono::Utc::now(),
            attached_machine_id: None,
            attached_allocation_id: None,
            block_size: 4096,
            blocks: 2621440,
            blocks_avail: 2621440,
            blocks_free: 2621440,
            fstype: "ext4".to_string(),
            snapshot_retention: None,
        };
        
        // Store volume
        {
            let mut volumes = state.volumes.write().unwrap();
            volumes.insert(volume.id.clone(), volume.clone());
        }
        
        // Retrieve volume
        {
            let volumes = state.volumes.read().unwrap();
            let stored_volume = volumes.get(&volume.id).unwrap();
            assert_eq!(stored_volume.name, "test-volume");
            assert_eq!(stored_volume.size_gb, 10);
            assert_eq!(stored_volume.encrypted, true);
        }
        
        // Attach volume to machine
        let machine_id = "d1234567890123456".to_string();
        {
            let mut volumes = state.volumes.write().unwrap();
            if let Some(v) = volumes.get_mut(&volume.id) {
                v.attached_machine_id = Some(machine_id.clone());
            }
        }
        
        // Verify attachment
        {
            let volumes = state.volumes.read().unwrap();
            let attached_volume = volumes.get(&volume.id).unwrap();
            assert_eq!(attached_volume.attached_machine_id, Some(machine_id));
        }
    }
    
    #[test]
    fn test_lease_management() {
        let state = test_state();
        let machine_id = "d1234567890123456".to_string();
        
        // Create test lease
        let lease = Lease {
            nonce: Uuid::new_v4().to_string(),
            machine_id: machine_id.clone(),
            ttl_seconds: 60,
            description: Some("test lease".to_string()),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(60),
            owner: LeaseOwner {
                id: "test-owner".to_string(),
                kind: "user".to_string(),
            },
        };
        
        // Store lease
        {
            let mut leases = state.leases.write().unwrap();
            leases.insert(machine_id.clone(), lease.clone());
        }
        
        // Retrieve lease
        {
            let leases = state.leases.read().unwrap();
            let stored_lease = leases.get(&machine_id).unwrap();
            assert_eq!(stored_lease.nonce, lease.nonce);
            assert_eq!(stored_lease.ttl_seconds, 60);
        }
        
        // Check lease expiration
        {
            let leases = state.leases.read().unwrap();
            let active_lease = leases.get(&machine_id).unwrap();
            assert!(active_lease.expires_at > chrono::Utc::now());
        }
        
        // Release lease
        {
            let mut leases = state.leases.write().unwrap();
            leases.remove(&machine_id);
        }
        
        // Verify release
        {
            let leases = state.leases.read().unwrap();
            assert!(leases.get(&machine_id).is_none());
        }
    }
    
    #[test]
    fn test_event_tracking() {
        let state = test_state();
        
        // Add events
        {
            let mut events = state.events.write().unwrap();
            events.push(MachineEvent {
                id: Uuid::new_v4().to_string(),
                machine_id: Some("d1234567890123456".to_string()),
                app_name: Some("test-app".to_string()),
                event_type: "machine.started".to_string(),
                source: "api".to_string(),
                status: EventStatus::Completed,
                timestamp: chrono::Utc::now(),
                request: None,
            });
            
            events.push(MachineEvent {
                id: Uuid::new_v4().to_string(),
                machine_id: Some("d1234567890123456".to_string()),
                app_name: Some("test-app".to_string()),
                event_type: "machine.stopped".to_string(),
                source: "api".to_string(),
                status: EventStatus::Completed,
                timestamp: chrono::Utc::now(),
                request: None,
            });
        }
        
        // Check event count
        {
            let events = state.events.read().unwrap();
            assert_eq!(events.len(), 2);
        }
        
        // Filter events by machine
        {
            let events = state.events.read().unwrap();
            let machine_events: Vec<_> = events.iter()
                .filter(|e| e.machine_id == Some("d1234567890123456".to_string()))
                .collect();
            assert_eq!(machine_events.len(), 2);
        }
        
        // Clear old events (simulate cleanup)
        {
            let mut events = state.events.write().unwrap();
            let cutoff_time = chrono::Utc::now() - chrono::Duration::hours(1);
            events.retain(|e| e.timestamp > cutoff_time);
            // All events should be retained as they're recent
            assert_eq!(events.len(), 2);
        }
    }
    
    #[test]
    fn test_concurrent_access() {
        use std::thread;
        use std::sync::Arc;
        
        let state = Arc::new(test_state());
        let mut handles = vec![];
        
        // Spawn multiple threads to test concurrent access
        for i in 0..10 {
            let state_clone = Arc::clone(&state);
            let handle = thread::spawn(move || {
                // Create and store app
                let app = App {
                    id: Uuid::new_v4().to_string(),
                    name: format!("concurrent-app-{}", i),
                    organization: Organization {
                        id: "test-org".to_string(),
                        slug: "test-org".to_string(),
                        name: "Test Organization".to_string(),
                    },
                    status: AppStatus::Running,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };
                
                // Write operation
                {
                    let mut apps = state_clone.apps.write().unwrap();
                    apps.insert(app.name.clone(), app);
                }
                
                // Read operation
                {
                    let apps = state_clone.apps.read().unwrap();
                    assert!(apps.contains_key(&format!("concurrent-app-{}", i)));
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify all apps were created
        {
            let apps = state.apps.read().unwrap();
            assert_eq!(apps.len(), 10);
            for i in 0..10 {
                assert!(apps.contains_key(&format!("concurrent-app-{}", i)));
            }
        }
    }
}