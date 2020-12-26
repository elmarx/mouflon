use crate::config::{ClientConfig, Endpoint};
use crate::model::AccessTokenResponse;
use crate::BoxResult;
use hyper::client::HttpConnector;
use hyper::Client;
use hyper_tls::HttpsConnector;

pub struct Oauth2Client {
    client: Client<HttpsConnector<HttpConnector>>,
    authorization_endpoint: String,
    token_endpoint: String,
    client_id: String,
    client_secret: Option<String>,
}

impl Oauth2Client {}

impl Oauth2Client {
    pub async fn from(config: &ClientConfig) -> Self {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);

        match &config.endpoint {
            Endpoint::Issuer(_iss) => todo!("get token endpoint via oidc discovery"),
        }
    }

    pub async fn refresh_token(&self, rt: &str) -> BoxResult<AccessTokenResponse> {
        todo!("implement refresh token flow: {}", rt)
    }

    pub async fn access_token(&self) -> BoxResult<AccessTokenResponse> {
        todo!("execute access_token flow")
    }
}
