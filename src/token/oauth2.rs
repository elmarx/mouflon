use crate::config::{ClientConfig, Endpoint};
use crate::model::AccessTokenResponse;
use crate::token::openid_configuration::OpenIdConfiguration;
use crate::BoxResult;
use bytes::buf::BufExt;
use hyper::body::aggregate;
use hyper::client::HttpConnector;
use hyper::header::CONTENT_TYPE;
use hyper::{Body, Client, Request, Response};
use hyper_tls::HttpsConnector;
use mime::APPLICATION_WWW_FORM_URLENCODED;

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

    async fn token_request(&self, body: String) -> BoxResult<Response<Body>> {
        let body = Body::from(body);

        let req = Request::builder()
            .method("POST")
            .uri(&*self.token_endpoint)
            .header(CONTENT_TYPE, APPLICATION_WWW_FORM_URLENCODED.as_ref())
            .body(body)?;

        Ok(self.client.request(req).await?)
    }

    pub async fn refresh_token(&self, rt: &str) -> BoxResult<AccessTokenResponse> {
        let req_body = format!(
            "grant_type=refresh_token&client_id={}&client_secret={}&refresh_token={}",
            self.client_id,
            self.client_secret
                .as_ref()
                .map_or_else(|| "", |s| s.as_str()),
            rt
        );

        let response = self.token_request(req_body).await?;
        let body = aggregate(response).await?;

        Ok(serde_json::from_reader(body.reader())?)
    }

    pub async fn access_token(&self) -> BoxResult<AccessTokenResponse> {
        todo!("execute access_token flow")
    }
}
