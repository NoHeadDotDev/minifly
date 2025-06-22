//! Stop platform command

use anyhow::Result;
use colored::*;

/// Handle the stop command
pub async fn handle(force: bool) -> Result<()> {
    println!("{}", "🛑 Stopping Minifly platform...".yellow().bold());
    
    if force {
        println!("  • Force mode enabled");
    }
    
    println!();
    println!("{}", "💡 Platform control not available in standalone CLI".yellow());
    println!("   Install the full platform for complete platform management");
    
    Ok(())
}