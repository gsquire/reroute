extern crate hyper;
extern crate reroute;

use std::io::Read;

use hyper::Server;
use hyper::server::{Request, Response};
use reroute::{RouterBuilder, Captures};

fn digit_handler(_: Request, res: Response, c: Captures) {
    // We know there are captures because that is the only way this function is triggered.
    let caps = c.unwrap();
    let digits = &caps[1];
    if digits.len() > 5 {
        res.send(b"that's a big number!").unwrap();
    } else {
        res.send(b"not a big number").unwrap();
    }
}

// You can ignore captures if you don't want to use them.
fn body_handler(mut req: Request, res: Response, _: Captures) {
    println!("request from {}", req.remote_addr);

    // Read the request body into a string and then print it back out on the response.
    let mut body = String::new();
    let _ = req.read_to_string(&mut body);
    res.send(body.as_bytes()).unwrap();
}

// A custom 404 handler.
fn not_found(req: Request, res: Response, _: Captures) {
    let uri = format!("{}", req.uri);
    let message = format!("why you calling {}?", uri);
    res.send(message.as_bytes()).unwrap();
}

fn main() {
    let mut builder = RouterBuilder::new();

    // Use raw strings so you don't need to escape patterns.
    builder.get(r"/(\d+)", digit_handler);
    builder.post(r"/body", body_handler);

    // Using a closure also works!
    builder.delete(r"/closure", |_: Request, res: Response, _: Captures| {
        res.send(b"You used a closure here, and called a delete. How neat.").unwrap();
    });

    // Add your own not found handler.
    builder.not_found(not_found);

    let router = builder.finalize().unwrap();

    // You can pass the router to hyper's Server's handle function as it
    // implements the Handle trait.
    Server::http("127.0.0.1:3000").unwrap().handle(router).unwrap();
}
