use thalos_core::module::ModuleEntry;

pub mod ping;
pub mod user;

pub fn all_modules() -> Vec<ModuleEntry> {
    vec![
        ping::PingModule.into(),
        user::UserModule.into(),
        comment_module::CommentModule.into(),
    ]
}
