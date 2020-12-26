use crate::BoxResult;
use bytes::buf::BufExt;
use hyper::client::HttpConnector;
use hyper::{Client, Uri};
use hyper_tls::HttpsConnector;
use serde::Deserialize;

/// data provided via [openid connect discovery](https://openid.net/specs/openid-connect-discovery-1_0.html)
#[derive(Deserialize, Debug)]
pub struct OpenIdConfiguration {
    pub token_endpoint: String,
    pub authorization_endpoint: String,
}

impl OpenIdConfiguration {
    pub async fn from_iss_with_client(
        client: &Client<HttpsConnector<HttpConnector>>,
        iss: &str,
    ) -> BoxResult<Self> {
        let uri: Uri = format!("{}/.well-known/openid-configuration", iss).parse()?;
        let res = client.get(uri).await?;
        let body = hyper::body::aggregate(res).await?;

        let config = serde_json::from_reader(body.reader())?;

        Ok(config)
    }
}
