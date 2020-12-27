use crate::BoxResult;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;

pub(crate) struct ReturnParams {
    pub code: String,
    pub state: String,
}

/// start a webserver and return the query-parameter "code"
pub(crate) async fn receive_code() -> BoxResult<ReturnParams> {
    let (tx, rx) = tokio::sync::oneshot::channel::<ReturnParams>();
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
                        let code = params.get("code").unwrap().to_owned();
                        let state = params.get("state").unwrap().to_owned();
                        let _ = tx.send(ReturnParams { code, state });
                    }

                    Ok::<_, Infallible>(Response::new(Body::from("I may have received the code.")))
                }
            }))
        }
    });

    let addr = ([127, 0, 0, 1], 4800).into();

    let server = Server::bind(&addr).serve(make_svc);

    let mut return_params: Option<ReturnParams> = None;
    let graceful = server.with_graceful_shutdown(async {
        return_params = rx.await.ok();
    });

    graceful.await?;

    Ok(return_params.unwrap())
}
