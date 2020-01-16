use std::convert::Infallible;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use lazy_static::lazy_static;
use reroute::{Captures, RouterBuilder};

lazy_static! {
    static ref ROUTER: reroute::Router = {
        let mut builder = RouterBuilder::new();

        // Use raw strings so you don't need to escape patterns.
        builder.get(r"/(\d+)", digit_handler);
        builder.post(r"/body", body_handler);

        // Using a closure also works!
        builder.delete(r"/closure", |_: Request<Body>, _: Captures| {
            Response::new(Body::from(
                "You used a closure here, and called a delete. How neat.",
            ))
        });

        // Add your own not found handler.
        builder.not_found(not_found);

        builder.finalize().unwrap()
    };
}

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
    Response::new(req.into_body())
}

// A custom 404 handler.
fn not_found(req: Request<Body>, _: Captures) -> Response<Body> {
    let message = format!("why you calling {}?", req.uri());
    Response::new(Body::from(message))
}

async fn handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(ROUTER.handle(req))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 3000).into();
    let svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handler)) });
    let server = Server::bind(&addr).serve(svc);

    server.await?;

    Ok(())
}
