use lazy_static::lazy_static;
use crate::config::connection::ConnectionConfig;
use crate::config::get_config;

mod config;

lazy_static! {
    static ref CONNECTION_SETTING: ConnectionConfig = get_config("connection");
}

fn main() {
    println!("Hello, world!");
    println!("{:?}", CONNECTION_SETTING.ssl_key);
}
