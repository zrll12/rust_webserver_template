use std::fs::{create_dir_all, OpenOptions};
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
use tracing::info;

pub mod core;

pub fn get_config<T>(name: &str) -> T
where T: for<'a> Deserialize<'a> + Serialize
{
    let file_name = format!("config/{}.toml", name);
    let mut raw_config = String::new();

    create_dir_all("config").unwrap();
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&file_name)
        .expect(format!("Cannot open {}", &file_name).as_str());
    file.read_to_string(&mut raw_config).unwrap();

    let config: T = toml::from_str(&raw_config).unwrap();
    save_config(name, &config);
    
    info!("Config loaded: {}", &file_name);

    config
}

pub fn save_config<T>(name: &str, config: &T)
where T: Serialize
{
    let file_name = format!("config/{}.toml", name);
    let config_str = toml::to_string_pretty(config).unwrap();

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&file_name)
        .expect(format!("Cannot open {}", &file_name).as_str());
    file.write_all(config_str.as_bytes()).unwrap();
}