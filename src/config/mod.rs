mod keycloak;
mod read_configs;
pub use read_configs::read_configs;

#[derive(Debug)]
pub enum Endpoint {
    Issuer(String),
    /*
    List {
        authorization: String,
        token: String,
    },
     */
}

#[derive(Debug)]
pub struct ClientConfig {
    pub name: String,
    pub endpoint: Endpoint,
    pub client_id: String,
    pub client_secret: Option<String>,
}
