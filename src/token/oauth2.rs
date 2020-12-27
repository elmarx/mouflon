use crate::browser::open_url_in_browser;
use crate::config::{ClientConfig, Endpoint};
use crate::model::AccessTokenResponse;
use crate::token::openid_configuration::OpenIdConfiguration;
use crate::token::pkce::{url_safe_rand, verifier_to_challenge};
use crate::token::receive_code::receive_code;
use crate::BoxResult;
use bytes::buf::BufExt;
use hyper::body::aggregate;
use hyper::client::HttpConnector;
use hyper::header::CONTENT_TYPE;
use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;
use log::error;
use mime::APPLICATION_WWW_FORM_URLENCODED;
use serde::de::DeserializeOwned;
use serde_json::Value;

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

    fn client_secret_str(&self) -> &str {
        self.client_secret
            .as_ref()
            .map_or_else(|| "", |s| s.as_str())
    }

    async fn token_request<T: DeserializeOwned>(&self, body: String) -> BoxResult<T> {
        let body = Body::from(body);

        let req = Request::builder()
            .method("POST")
            .uri(&*self.token_endpoint)
            .header(CONTENT_TYPE, APPLICATION_WWW_FORM_URLENCODED.as_ref())
            .body(body)?;

        let response = self.client.request(req).await?;
        let body = aggregate(response).await?;

        let json: Value = serde_json::from_reader(body.reader())?;

        match T::deserialize(&json) {
            Ok(v) => Ok(v),
            Err(err) => {
                error!("error reading response {}: {:?}", json, err);
                Err(Box::new(err))
            }
        }
    }

    pub async fn refresh_token(&self, rt: &str) -> BoxResult<AccessTokenResponse> {
        let req_body = format!(
            "grant_type=refresh_token&client_id={}&client_secret={}&refresh_token={}",
            self.client_id,
            self.client_secret_str(),
            rt
        );

        self.token_request(req_body).await
    }

    pub async fn access_token(&self) -> BoxResult<AccessTokenResponse> {
        let stored_state = url_safe_rand(16);
        let code_verifier = url_safe_rand(64);
        let code_challenge = verifier_to_challenge(&*code_verifier);
        let redirect_uri = "http://localhost:4800/";

        let auth_url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&code_challenge={}&code_challenge_method=S256&state={}",
            self.authorization_endpoint,
            self.client_id,
            redirect_uri,
            code_challenge,
            stored_state
        );

        open_url_in_browser(&*auth_url)?;

        let returned = receive_code().await?;
        if returned.state != stored_state {
            todo!("handle CSRF issue")
        }

        let auth_code_params = format!(
            "grant_type=authorization_code&client_id={}&client_secret={}&redirect_uri={}&code={}&code_verifier={}",
            self.client_id,
            self.client_secret_str(),
            redirect_uri,
            returned.code,
            code_verifier
        );

        Ok(self.token_request(auth_code_params).await?)
    }
}
