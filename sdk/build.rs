use std::fs;
use std::path::Path;

fn main() {
    // Tell Cargo to rerun this build script if the .env file changes
    println!("cargo:rerun-if-changed=.env");

    // Look for .env file in the parent directory (project root)
    let env_path = Path::new(".env");

    if env_path.exists() {
        // Read the .env file
        let contents = fs::read_to_string(env_path).expect("Failed to read .env file");

        // Parse each line and set environment variables for compilation
        for line in contents.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse KEY=VALUE format
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                // Remove quotes if present
                let value = if (value.starts_with('"') && value.ends_with('"'))
                    || (value.starts_with('\'') && value.ends_with('\''))
                {
                    &value[1..value.len() - 1]
                } else {
                    value
                };

                // Set the environment variable for compile time
                println!("cargo:rustc-env={}={}", key, value);
            }
        }
    } else {
        println!("cargo:warning=.env file not found at {:?}", env_path);
    }
}
