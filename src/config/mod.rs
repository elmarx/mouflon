mod keycloak;

pub enum Endpoint {
    Issuer(String),
    /*
    List {
        authorization: String,
        token: String,
    },
     */
}

pub struct ClientConfig {
    pub name: String,
    pub endpoint: Endpoint,
    pub client_id: String,
    pub client_secret: Option<String>,
}