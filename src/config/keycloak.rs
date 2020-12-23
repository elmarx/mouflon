use crate::config::ClientConfig;
use crate::config::Endpoint::Issuer;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct KeycloakClientConfig {
    pub resource: String,
    pub credentials: Option<KeycloakClientCredentials>,
    pub realm: String,
    pub auth_server_url: String,
    #[serde(default)]
    pub public_client: bool,
}

#[derive(Deserialize, Debug)]
pub struct KeycloakClientCredentials {
    pub secret: String,
}

impl KeycloakClientConfig {
    pub fn into_client_config(self, name: String) -> ClientConfig {
        ClientConfig {
            name,
            client_id: self.resource,
            client_secret: self.credentials.map(|c| c.secret),
            endpoint: Issuer(format!("{}realms/{}", self.auth_server_url, self.realm)),
        }
    }
}
