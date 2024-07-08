use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

#[serde_inline_default]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConnectionConfig {
    #[serde_inline_default(String::from("0.0.0.0:7890"))]
    pub server_addr: String,
    #[serde_inline_default(String::from("postgresql://root:root@127.0.0.1:5432/test"))]
    pub db_uri: String,
    #[serde_inline_default(false)]
    pub tls: bool,
    #[serde_inline_default(String::from("./cert.crt"))]
    pub ssl_cert: String,
    #[serde_inline_default(String::from("./private.key"))]
    pub ssl_key: String
}

