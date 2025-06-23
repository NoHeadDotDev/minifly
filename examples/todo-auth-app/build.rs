use std::env;
use std::path::PathBuf;

fn main() {
    // Ensure DATABASE_URL is set for SQLx compile-time checking
    if env::var("DATABASE_URL").is_err() {
        let out_dir = env::var("OUT_DIR").unwrap();
        let db_path = PathBuf::from(out_dir).join("app.db");
        env::set_var("DATABASE_URL", format!("sqlite:{}", db_path.display()));
    }
    
    println!("cargo:rerun-if-changed=migrations/");
}