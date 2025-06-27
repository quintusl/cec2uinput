#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

use anyhow::Result;
use serde::Deserialize;
use serde_yaml;
use std::fs::File;

use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct Config {
    _vendor_id: u16,
    _product_id: u16,
    mappings: HashMap<String, String>,
}

fn main() -> Result<()> {
    let config = load_config("config.yaml")?;

    #[cfg(target_os = "linux")]
    linux::run(&config)?;

    #[cfg(target_os = "macos")]
    macos::run(&config)?;

    Ok(())
}

fn load_config(path: &str) -> Result<Config> {
    let file = File::open(path)?;
    let config: Config = serde_yaml::from_reader(file)?;
    Ok(config)
}
