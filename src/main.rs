use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;
mod config;

/// start a webserver and return the query-parameter "code"
pub async fn receive_code() -> String {
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    let tx = Arc::new(Mutex::new(Some(tx)));

    let make_svc = make_service_fn(move |_conn| {
        let tx = tx.clone();

        async move {
            let tx = tx.clone();

            Ok::<_, Infallible>(service_fn(move |request: Request<Body>| {
                let tx = tx.clone();

                let params: HashMap<String, String> =
                    request.uri().query().map_or_else(HashMap::new, |v| {
                        url::form_urlencoded::parse(v.as_bytes())
                            .into_owned()
                            .collect()
                    });

                async move {
                    let tx = tx.clone();

                    if let Some(tx) = tx.lock().await.take() {
                        let _ = tx.send(params.get("code").unwrap().to_owned());
                    }

                    Ok::<_, Infallible>(Response::new(Body::from("I may have received the code.")))
                }
            }))
        }
    });

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr).serve(make_svc);

    let mut code: Option<String> = None;
    let graceful = server.with_graceful_shutdown(async {
        code = rx.await.ok();
    });

    graceful.await.unwrap();

    code.unwrap()
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let code = receive_code().await;

    println!("{}", code);

    Ok(())
}
