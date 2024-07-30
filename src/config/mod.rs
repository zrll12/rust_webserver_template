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
    save_changes(name, &raw_config, &config);
    
    info!("Config loaded: {}", &file_name);

    config
}

pub fn save_changes<T>(name: &str, old_config: &str, config: &T)
where T: Serialize
{
    let file_name = format!("config/{}.toml", name);
    let new_config = toml::to_string_pretty(config).unwrap();

    let mut document = String::new();

    for diff in diff::lines(old_config, &new_config) {
        match diff {
            diff::Result::Left(l) => {
                if l.trim_start().starts_with("#") || l.is_empty() {
                    // keep comments or empty lines
                    document.push_str(l);
                    document.push('\n');
                    continue;
                }
            },
            diff::Result::Both(l, _) => {
                document.push_str(l);
                document.push('\n');
                continue;
            },
            diff::Result::Right(r) => {
                document.push_str(r);
                document.push('\n');
                continue;
            },
        }
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&file_name)
        .expect(format!("Cannot open {}", &file_name).as_str());
    file.write_all(document.trim_end().as_bytes()).unwrap();
}