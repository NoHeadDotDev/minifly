use anyhow::Result;
use colored::*;
use tabled::{Table, Tabled};
use crate::client::ApiClient;

#[derive(Tabled)]
struct MachineRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "STATE")]
    state: String,
    #[tabled(rename = "REGION")]
    region: String,
    #[tabled(rename = "IMAGE")]
    image: String,
    #[tabled(rename = "IP")]
    ip: String,
}

pub async fn list(client: &ApiClient, app: &str) -> Result<()> {
    let machines = client.list_machines(app).await?;
    
    if machines.is_empty() {
        println!("No machines found for app '{}'. Create one with: minifly machines create", app);
        return Ok(());
    }
    
    let rows: Vec<MachineRow> = machines.into_iter()
        .map(|m| MachineRow {
            id: m.id,
            name: m.name,
            state: format!("{:?}", m.state).to_lowercase(),
            region: m.region,
            image: format!("{}:{}", m.image_ref.repository, m.image_ref.tag),
            ip: m.private_ip,
        })
        .collect();
    
    let table = Table::new(rows);
    println!("{}", table);
    
    Ok(())
}

pub async fn create(
    client: &ApiClient,
    app: &str,
    image: &str,
    name: Option<String>,
    region: Option<String>,
) -> Result<()> {
    println!("Creating machine for app {}...", app.yellow());
    
    let machine = client.create_machine(app, image, name, region).await?;
    
    println!("{}", "Machine created successfully!".green());
    println!("ID: {}", machine.id);
    println!("Name: {}", machine.name);
    println!("State: {:?}", machine.state);
    println!("Region: {}", machine.region);
    println!("Private IP: {}", machine.private_ip);
    
    Ok(())
}

pub async fn start(client: &ApiClient, machine_id: &str) -> Result<()> {
    println!("Starting machine {}...", machine_id.yellow());
    
    let app = client.get_machine_app(machine_id).await?;
    let resp = client.start_machine(&app, machine_id).await?;
    
    println!("{}", "Machine started successfully!".green());
    println!("Previous state: {}", resp.previous_state);
    
    Ok(())
}

pub async fn stop(client: &ApiClient, machine_id: &str) -> Result<()> {
    println!("Stopping machine {}...", machine_id.yellow());
    
    let app = client.get_machine_app(machine_id).await?;
    let resp = client.stop_machine(&app, machine_id).await?;
    
    if resp.ok {
        println!("{}", "Machine stopped successfully!".green());
    }
    
    Ok(())
}

pub async fn delete(client: &ApiClient, machine_id: &str, force: bool) -> Result<()> {
    use dialoguer::Confirm;
    
    if !force {
        let confirm = Confirm::new()
            .with_prompt(format!("Are you sure you want to delete machine '{}'?", machine_id))
            .interact()?;
        
        if !confirm {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }
    
    println!("Deleting machine {}...", machine_id.yellow());
    
    let app = client.get_machine_app(machine_id).await?;
    client.delete_machine(&app, machine_id, force).await?;
    
    println!("{}", "Machine deleted successfully!".green());
    
    Ok(())
}