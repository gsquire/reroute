extern crate futures;
extern crate hyper;
extern crate reroute;

use futures::{future, Future};
use hyper::service::{service_fn, Service};
use hyper::{Body, Request, Response, Server};
use reroute::{Captures, RouterBuilder};

fn digit_handler(_: Request<Body>, c: Captures) -> Response<Body> {
    // We know there are captures because that is the only way this function is triggered.
    let caps = c.unwrap();
    let digits = &caps[1];
    if digits.len() > 5 {
        Response::new(Body::from("that's a big number!"))
    } else {
        Response::new(Body::from("not a big number"))
    }
}

// You can ignore captures if you don't want to use them.
fn body_handler(req: Request<Body>, _: Captures) -> Response<Body> {
    // Read the request body into a string and then print it back out on the response.
    Response::new(Body::from(req.into_body()))
}

// A custom 404 handler.
fn not_found(req: Request<Body>, _: Captures) -> Response<Body> {
    let uri = format!("{}", req.uri());
    let message = format!("why you calling {}?", uri);
    Response::new(Body::from(message))
}

fn main() {
    let mut builder = RouterBuilder::new();

    // Use raw strings so you don't need to escape patterns.
    builder.get(r"/(\d+)", digit_handler);
    builder.post(r"/body", body_handler);

    // Using a closure also works!
    builder.delete(
        r"/closure",
        |_: Request<Body>, _: Captures| -> Response<Body> {
            Response::new(Body::from(
                "You used a closure here, and called a delete. How neat.",
            ))
        },
    );

    // Add your own not found handler.
    builder.not_found(not_found);

    let addr = "127.0.0.1:3000".parse().unwrap();

    hyper::rt::run(future::lazy(move || {
        let router = builder.finalize().unwrap();

        let new_service = move || {
            let router = router.clone();
            service_fn(move |req| {
                // TODO: Oh no...
                let mut r = router.clone();
                r.call(req)
            })
        };

        let server = Server::bind(&addr)
            .serve(new_service)
            .map_err(|e| eprintln!("server error: {}", e));

        server
    }));
}
