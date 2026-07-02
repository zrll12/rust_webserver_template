use crate::module::ModuleEntry;

pub mod core;
pub mod ping;

pub fn all_modules() -> Vec<ModuleEntry> {
    vec![
        ping::PingModule.into(),
        // custom prefix example: (ping::PingModule, "info").into(),
    ]
}
