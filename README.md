# reroute
[![Build Status](https://travis-ci.org/gsquire/reroute.svg?branch=master)](https://travis-ci.org/gsquire/reroute)

A router for Rust's hyper framework using regular expressions.

A simple example to demonstrate how to use the router:

```rust
extern crate hyper;
extern crate reroute;

use hyper::Server;
use hyper::method::Method;
use hyper::server::{Request, Response};
use reroute::{Captures, Router};

fn digit_handler(_: Request, res: Response, c: Captures) {
    println!("captures: {:?}", c);
    res.send(b"It works for digits!").unwrap();
}

fn main() {
    let mut router = Router::new();

    // Use raw strings so you don't need to escape patterns.
    router.get(r"/(\d+)", digit_handler);

    // There is no 404 handler added, so it will use the default defined in the
    // library.
    router.finalize().unwrap();

    // You can pass the router to hyper's Server's handle function as it
    // implements the Handle trait.
    Server::http("127.0.0.1:3000").unwrap().handle(router).unwrap();
}
```

You can then hit localhost on port 3000 to see the responses based on the routes
that you pass.

```sh
curl localhost:3000/123 ->
    captures: Some(["/123", "123"])
    It works for digits!

curl localhost:3000/faux ->
    No route found for /faux
```
