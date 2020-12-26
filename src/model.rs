use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use serde_with::DurationSeconds;
use std::time::Duration;

/// the typical access-token response, returned for a valid authorization request
/// contains only properties relevant for mouflon (i.e. might be extended in the future)
#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct AccessTokenResponse {
    pub access_token: String,
    #[serde_as(as = "DurationSeconds<u64>")]
    pub expires_in: Duration,
    #[serde_as(as = "DurationSeconds<u64>")]
    pub refresh_expires_in: Duration,
    pub refresh_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthorizationData {
    // "client side" iat, as the server-time might differ, and also we do not need to read the JWT
    pub iat: DateTime<Utc>,
    #[serde(alias = "atResponse")]
    pub at_response: AccessTokenResponse,
}

#[derive(Serialize, Debug)]
pub struct BorrowedAuthorizationData<'a> {
    pub iat: DateTime<Utc>,
    #[serde(alias = "atResponse")]
    pub at_response: &'a AccessTokenResponse,
}

impl<'a> BorrowedAuthorizationData<'a> {
    pub fn new(at_response: &'a AccessTokenResponse) -> Self {
        Self {
            iat: Utc::now(),
            at_response,
        }
    }
}
