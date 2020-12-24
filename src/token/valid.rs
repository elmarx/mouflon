use crate::model::AuthorizationData;
use chrono::{Duration as OldDuration, Utc};
use lazy_static::lazy_static;

lazy_static! {
    // grace period before expiration when to refetch the token (so it's not expired right before we print it)
    static ref VALID_GRACE_SECONDS: OldDuration = OldDuration::seconds(10);
}

pub(super) fn is_at_valid(auth: &AuthorizationData) -> bool {
    let now = Utc::now();
    let expire_date = auth.iat
        + (OldDuration::from_std(auth.at_response.expires_in).unwrap() - *VALID_GRACE_SECONDS);
    now < expire_date
}

pub(super) fn is_rt_valid(auth: &AuthorizationData) -> bool {
    let now = Utc::now();
    let expire_date = auth.iat
        + (OldDuration::from_std(auth.at_response.refresh_expires_in).unwrap()
            - *VALID_GRACE_SECONDS);

    now < expire_date
}
