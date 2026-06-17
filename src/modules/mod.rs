use thalos_core::module::AppModule;

pub mod ping;

#[cfg(all(test, feature = "openapi"))]
mod openapi_export;

pub fn all_modules() -> Vec<Box<dyn AppModule>> {
    vec![
        Box::new(ping::PingModule),
    ]
}
