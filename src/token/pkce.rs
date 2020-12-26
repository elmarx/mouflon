use rand::Rng;
use sha2::{Digest, Sha256};
use std::iter::from_fn;

// possible chars for pkce [code verifier](https://tools.ietf.org/html/rfc7636#section-4.1): unreserved URI characters
const URL_SAFE_CHARACTERS: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";

pub(super) fn url_safe_rand(len: usize) -> String {
    let mut rng = rand::thread_rng();

    from_fn(move || {
        let idx = rng.gen_range(0..URL_SAFE_CHARACTERS.len());
        Some(URL_SAFE_CHARACTERS[idx] as char)
    })
    .take(len)
    .collect()
}

/// build the challenge for a given verifier, i.e.: base64url-encoded sha256 hash
pub fn verifier_to_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier);
    let hash = hasher.finalize();
    base64_url::encode(hash.as_slice())
}

#[cfg(test)]
mod test {
    use crate::token::pkce::verifier_to_challenge;

    #[test]
    fn test_verifier_to_challenge() {
        // sample/result taken from https://www.oauth.com/playground/authorization-code-with-pkce.html
        let sample_verifier = "8KiKcKzgi3MQ08g_AYs1jkU8jFWBFiMXf8K4GPuJjOMrjozl";
        let expected = "s83bY2yOhn9C5nu_bJnw_O33lsaF0MK_l5R78gNLrcE";
        let actual = verifier_to_challenge(sample_verifier);
        assert_eq!(&*actual, expected);
    }
}
