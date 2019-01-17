use futures::future;
use futures::future::Future;
use futures::stream::Stream;

use tokio;

use hyper::client::connect::HttpConnector;
use hyper::Body;
use hyper::Client;
use hyper::Method;
use hyper::Request;

use hyper_tls::HttpsConnector;

mod error;

use crate::error::*;

fn derp(
    client: &Client<HttpsConnector<HttpConnector>>,
) -> impl Future<Item = (), Error = crate::error::Error> {
    let mut http_request = Request::new(Body::empty());

    let uri = "https://example.com/".to_string();

    let request_uri = match uri.parse() {
        Err(e) => {
            return future::Either::A(future::err(
                format!("error parsing request uri: {:}", e).into(),
            ))
        }
        Ok(request_uri) => request_uri,
    };

    *http_request.method_mut() = Method::GET;
    *http_request.uri_mut() = request_uri;

    // In the unused future below either first .then(...) or .and_then(...) need to be absent.
    //
    // * If the first .then(...) is missing, everything compiles just fine.
    //
    // * If .and_then(...) is missing, everything compiles just fine.
    //
    // * If both first .then(...) and .and_then(...) are present, the following happens:
    // --------------------------------------------------------------------------------------------
    // $ cargo run
    //    Compiling rust-hyper-error-chain-weirness v0.1.0 (/Users/bobrik/projects/rust-hyper-error-chain-weirness)
    // error[E0271]: type mismatch resolving `<futures::stream::concat::Concat2<hyper::body::body::Body> as futures::future::IntoFuture>::Error == error::Error`
    //   --> src/main.rs:85:10
    //    |
    // 85 |         .and_then(|http_response| http_response.into_body().concat2())
    //    |          ^^^^^^^^ expected struct `hyper::error::Error`, found struct `error::Error`
    //    |
    //    = note: expected type `hyper::error::Error`
    //               found type `error::Error`
    //
    // error[E0599]: no method named `then` found for type `futures::future::and_then::AndThen<futures::future::then::Then<hyper::client::ResponseFuture, futures::future::result_::FutureResult<http::response::Response<hyper::body::body::Body>, error::Error>, [closure@src/main.rs:84:15: 84:55]>, futures::stream::concat::Concat2<hyper::body::body::Body>, [closure@src/main.rs:85:19: 85:70]>` in the current scope
    //   --> src/main.rs:86:10
    //    |
    // 86 |         .then(|_| future::result::<(), ()>(Ok(())));
    //    |          ^^^^
    //    |
    //    = note: the method `then` exists but the following trait bounds were not satisfied:
    //            `&mut futures::future::and_then::AndThen<futures::future::then::Then<hyper::client::ResponseFuture, futures::future::result_::FutureResult<http::response::Response<hyper::body::body::Body>, error::Error>, [closure@src/main.rs:84:15: 84:55]>, futures::stream::concat::Concat2<hyper::body::body::Body>, [closure@src/main.rs:85:19: 85:70]> : futures::future::Future`
    //            `&mut futures::future::and_then::AndThen<futures::future::then::Then<hyper::client::ResponseFuture, futures::future::result_::FutureResult<http::response::Response<hyper::body::body::Body>, error::Error>, [closure@src/main.rs:84:15: 84:55]>, futures::stream::concat::Concat2<hyper::body::body::Body>, [closure@src/main.rs:85:19: 85:70]> : futures::stream::Stream`
    //            `futures::future::and_then::AndThen<futures::future::then::Then<hyper::client::ResponseFuture, futures::future::result_::FutureResult<http::response::Response<hyper::body::body::Body>, error::Error>, [closure@src/main.rs:84:15: 84:55]>, futures::stream::concat::Concat2<hyper::body::body::Body>, [closure@src/main.rs:85:19: 85:70]> : futures::future::Future`
    //
    // error: aborting due to 2 previous errors
    //
    // Some errors occurred: E0271, E0599.
    // For more information about an error, try `rustc --explain E0271`.
    // error: Could not compile `rust-hyper-error-chain-weirness`.
    //
    // To learn more, run the command again with --verbose.
    // --------------------------------------------------------------------------------------------
    //
    // The question is why does .and_then() require an error from hyper, when its argument
    // is a non-result type that does not include error at all: hyper::Response<Body>.

    client
        .request(http_request)
        .then(|r| future::result(r.chain_err(|| "eh")))
        .and_then(|http_response| http_response.into_body().concat2())
        .then(|_| future::result::<(), ()>(Ok(())));

    future::Either::B(future::ok(()))
}

fn main() {
    let connector = HttpsConnector::new(4).chain_err(|| "unable to create http connector").unwrap();
    let client = Client::builder().build::<_, Body>(connector);

    tokio::run(derp(&client).map_err(|e| println!("life happened: {}", e)));
}
