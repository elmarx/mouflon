use crate::config::{ClientConfig, Endpoint};
use crate::model::AccessTokenResponse;
use crate::token::openid_configuration::OpenIdConfiguration;
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
    pub async fn from(config: &ClientConfig) -> BoxResult<Self> {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);

        let (token_endpoint, authorization_endpoint) = match &config.endpoint {
            Endpoint::Issuer(iss) => {
                let oidc_config = OpenIdConfiguration::from_iss_with_client(&client, iss).await?;

                (
                    oidc_config.token_endpoint,
                    oidc_config.authorization_endpoint,
                )
            }
        };

        Ok(Self {
            client,
            authorization_endpoint,
            token_endpoint,
            client_id: config.client_id.clone(),
            client_secret: config.client_secret.clone(),
        })
    }

    pub async fn refresh_token(&self, rt: &str) -> BoxResult<AccessTokenResponse> {
        todo!("implement refresh token flow: {}", rt)
    }

    pub async fn access_token(&self) -> BoxResult<AccessTokenResponse> {
        todo!("execute access_token flow")
    }
}
