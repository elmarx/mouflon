use crate::config::{ClientConfig, Endpoint};
use crate::model::AccessTokenResponse;
use hyper::client::HttpConnector;
use hyper::Client;
use std::convert::Infallible;

pub struct Oauth2Client {
    client: Client<HttpConnector>,
    authorization_endpoint: String,
    token_endpoint: String,
    client_id: String,
    client_secret: Option<String>,
}

impl Oauth2Client {}

impl Oauth2Client {
    pub async fn from(config: &ClientConfig) -> Self {
        match &config.endpoint {
            Endpoint::Issuer(_iss) => todo!("get token endpoint via oidc discovery"),
        }
    }

    pub async fn refresh_token(&self, rt: &str) -> Result<AccessTokenResponse, Infallible> {
        todo!("implement refresh token flow: {}", rt)
    }

    pub async fn access_token(&self) -> Result<AccessTokenResponse, Infallible> {
        todo!("execute access_token flow")
    }
}
