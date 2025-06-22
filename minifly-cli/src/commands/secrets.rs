use anyhow::{Context, Result, bail};
use colored::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Loads secrets from .fly.secrets files for the specified application.
/// 
/// This function implements a hierarchical secrets loading system:
/// 1. First loads app-specific secrets from `.fly.secrets.<app_name>`
/// 2. Then loads default secrets from `.fly.secrets`
/// 3. App-specific secrets take precedence over default secrets
/// 
/// # Arguments
/// 
/// * `app_name` - The name of the application to load secrets for
/// 
/// # Returns
/// 
/// A HashMap containing all loaded secrets as key-value pairs.
/// 
/// # File Format
/// 
/// Secrets files use a simple KEY=VALUE format:
/// ```text
/// # Comments start with #
/// DATABASE_URL=postgres://localhost/myapp
/// SECRET_KEY=your-secret-key-here
/// API_TOKEN=sk-1234567890
/// ```
/// 
/// # Example
/// 
/// ```rust
/// # use minifly_cli::commands::secrets::load_secrets;
/// # tokio_test::block_on(async {
/// let secrets = load_secrets("myapp").await.unwrap();
/// println!("Loaded {} secrets", secrets.len());
/// # });
/// ```
pub async fn load_secrets(app_name: &str) -> Result<HashMap<String, String>> {
    let mut secrets = HashMap::new();
    
    // Try app-specific secrets file first
    let app_secrets_file = format!(".fly.secrets.{}", app_name);
    if Path::new(&app_secrets_file).exists() {
        let contents = fs::read_to_string(&app_secrets_file).await
            .context(format!("Failed to read {}", app_secrets_file))?;
        parse_secrets(&contents, &mut secrets)?;
    }
    
    // Then load default secrets file
    let default_secrets_file = ".fly.secrets";
    if Path::new(default_secrets_file).exists() {
        let contents = fs::read_to_string(default_secrets_file).await
            .context("Failed to read .fly.secrets")?;
        // Don't overwrite app-specific secrets
        let mut default_secrets = HashMap::new();
        parse_secrets(&contents, &mut default_secrets)?;
        for (k, v) in default_secrets {
            secrets.entry(k).or_insert(v);
        }
    }
    
    Ok(secrets)
}

/// Parses secrets from file contents in KEY=VALUE format.
/// 
/// Supports:
/// - Comments starting with #
/// - Empty lines (ignored)
/// - KEY=VALUE pairs with optional whitespace
/// - Values can contain = characters
/// 
/// # Arguments
/// 
/// * `contents` - The file contents to parse
/// * `secrets` - HashMap to store parsed secrets in
/// 
/// # Errors
/// 
/// Returns an error if:
/// - A line is not in KEY=VALUE format
/// - A key is empty or invalid
fn parse_secrets(contents: &str, secrets: &mut HashMap<String, String>) -> Result<()> {
    for (line_num, line) in contents.lines().enumerate() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Parse KEY=VALUE
        if let Some(pos) = line.find('=') {
            let key = line[..pos].trim();
            let value = line[pos + 1..].trim();
            
            if key.is_empty() {
                bail!("Empty key at line {}", line_num + 1);
            }
            
            // Remove quotes if present
            let value = if (value.starts_with('"') && value.ends_with('"')) ||
                        (value.starts_with('\'') && value.ends_with('\'')) {
                &value[1..value.len() - 1]
            } else {
                value
            };
            
            secrets.insert(key.to_string(), value.to_string());
        } else {
            bail!("Invalid format at line {} - expected KEY=VALUE", line_num + 1);
        }
    }
    
    Ok(())
}

/// Handle the secrets command
pub async fn handle(action: &str, args: Vec<String>) -> Result<()> {
    match action {
        "set" => handle_set(args).await,
        "list" => handle_list(args).await,
        "remove" => handle_remove(args).await,
        _ => bail!("Unknown secrets action: {}", action),
    }
}

/// Handle secrets set command
async fn handle_set(args: Vec<String>) -> Result<()> {
    if args.is_empty() {
        bail!("Usage: minifly secrets set KEY=VALUE [KEY2=VALUE2 ...]");
    }
    
    let app_name = get_app_name_from_fly_toml().await?;
    let secrets_file = format!(".fly.secrets.{}", app_name);
    
    // Load existing secrets
    let mut secrets = if Path::new(&secrets_file).exists() {
        load_secrets(&app_name).await?
    } else {
        HashMap::new()
    };
    
    // Parse and add new secrets
    for arg in &args {
        if let Some(pos) = arg.find('=') {
            let key = arg[..pos].trim();
            let value = arg[pos + 1..].trim();
            
            if key.is_empty() {
                bail!("Empty key in argument: {}", arg);
            }
            
            secrets.insert(key.to_string(), value.to_string());
            println!("✓ Set secret {}", key.green());
        } else {
            bail!("Invalid format: {} - expected KEY=VALUE", arg);
        }
    }
    
    // Write secrets file
    write_secrets_file(&secrets_file, &secrets).await?;
    
    println!("\n{} secrets set for app {}", args.len(), app_name.yellow());
    println!("Secrets are stored in {} (gitignored)", secrets_file.dimmed());
    
    Ok(())
}

/// Handle secrets list command
async fn handle_list(args: Vec<String>) -> Result<()> {
    let app_name = if args.is_empty() {
        get_app_name_from_fly_toml().await?
    } else {
        args[0].clone()
    };
    
    let secrets = load_secrets(&app_name).await?;
    
    if secrets.is_empty() {
        println!("No secrets found for app {}", app_name.yellow());
        println!("\nSet secrets with: minifly secrets set KEY=VALUE");
        return Ok(());
    }
    
    println!("Secrets for app {}:", app_name.yellow());
    println!();
    
    let mut keys: Vec<_> = secrets.keys().collect();
    keys.sort();
    
    for key in keys {
        println!("  {} = {}", key.green(), "<redacted>".dimmed());
    }
    
    println!("\n{} secrets total", secrets.len());
    
    Ok(())
}

/// Handle secrets remove command
async fn handle_remove(args: Vec<String>) -> Result<()> {
    if args.is_empty() {
        bail!("Usage: minifly secrets remove KEY [KEY2 ...]");
    }
    
    let app_name = get_app_name_from_fly_toml().await?;
    let secrets_file = format!(".fly.secrets.{}", app_name);
    
    if !Path::new(&secrets_file).exists() {
        bail!("No secrets file found for app {}", app_name);
    }
    
    let mut secrets = load_secrets(&app_name).await?;
    
    for key in &args {
        if secrets.remove(key).is_some() {
            println!("✓ Removed secret {}", key.red());
        } else {
            println!("⚠️  Secret {} not found", key.yellow());
        }
    }
    
    // Write updated secrets file
    write_secrets_file(&secrets_file, &secrets).await?;
    
    Ok(())
}

/// Write secrets to file
async fn write_secrets_file(path: &str, secrets: &HashMap<String, String>) -> Result<()> {
    let mut content = String::new();
    content.push_str("# Minifly secrets file - DO NOT COMMIT TO VERSION CONTROL\n");
    content.push_str("# Generated by minifly secrets command\n\n");
    
    let mut keys: Vec<_> = secrets.keys().collect();
    keys.sort();
    
    for key in keys {
        let value = &secrets[key];
        // Quote values that contain spaces or special characters
        let quoted_value = if value.contains(' ') || value.contains('"') || value.contains('\'') {
            format!("\"{}\"", value.replace('"', "\\\""))
        } else {
            value.clone()
        };
        content.push_str(&format!("{}={}\n", key, quoted_value));
    }
    
    let mut file = fs::File::create(path).await
        .context(format!("Failed to create {}", path))?;
    file.write_all(content.as_bytes()).await
        .context("Failed to write secrets file")?;
    
    Ok(())
}

/// Get app name from fly.toml
async fn get_app_name_from_fly_toml() -> Result<String> {
    let fly_toml = fs::read_to_string("fly.toml").await
        .context("Failed to read fly.toml - are you in a Fly app directory?")?;
    
    let config: toml::Value = toml::from_str(&fly_toml)
        .context("Failed to parse fly.toml")?;
    
    config.get("app")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("No app name found in fly.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_secrets() {
        let contents = r#"
# This is a comment
DATABASE_URL=postgres://localhost/mydb
API_KEY=abc123

# Another comment
SECRET_KEY="with spaces"
QUOTED='single quotes'
EMPTY=
"#;
        
        let mut secrets = HashMap::new();
        parse_secrets(contents, &mut secrets).unwrap();
        
        assert_eq!(secrets.get("DATABASE_URL").unwrap(), "postgres://localhost/mydb");
        assert_eq!(secrets.get("API_KEY").unwrap(), "abc123");
        assert_eq!(secrets.get("SECRET_KEY").unwrap(), "with spaces");
        assert_eq!(secrets.get("QUOTED").unwrap(), "single quotes");
        assert_eq!(secrets.get("EMPTY").unwrap(), "");
        assert_eq!(secrets.len(), 5);
    }
    
    #[test]
    fn test_parse_secrets_invalid_format() {
        let contents = "INVALID_LINE_NO_EQUALS";
        let mut secrets = HashMap::new();
        let result = parse_secrets(contents, &mut secrets);
        assert!(result.is_err());
    }
}