use ws_core::module::AppModule;

pub mod ping;
pub mod user;

#[cfg(all(test, feature = "openapi"))]
mod openapi_export;

pub fn all_modules() -> Vec<Box<dyn AppModule>> {
    vec![
        Box::new(ping::PingModule),
        Box::new(user::UserModule),
        Box::new(comment_module::CommentModule),
    ]
}
