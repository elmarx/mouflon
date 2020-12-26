use rand::Rng;
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
