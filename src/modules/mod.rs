use ws_core::module::AppModule;

pub mod ping;

pub fn all_modules() -> Vec<Box<dyn AppModule>> {
    vec![
        Box::new(ping::PingModule),
    ]
}
