use thalos_core::module::ModuleEntry;

pub mod ping;

pub fn all_modules() -> Vec<ModuleEntry> {
    vec![
        ping::PingModule.into(),
        // custom prefix example: (ping::PingModule, "info").into(),
    ]
}
