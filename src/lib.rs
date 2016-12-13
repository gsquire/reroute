extern crate hyper;
extern crate regex;

use hyper::method::Method;
use hyper::server::{Handler, Request, Response};
use hyper::status::StatusCode;
use regex::{Regex, RegexSet};

pub use error::Error;

mod error;

pub type Captures = Option<Vec<String>>;
type RouteHandler = Box<Fn(Request, Response, Captures) + Send + Sync>;

/// The Router struct contains the information for your app to route requests
/// properly based on their HTTP method and matching route. It allows the use
/// of a custom 404 handler if desired but provides a default as well.
///
/// Under the hood a Router uses a `RegexSet` to match URI's that come in to the
/// instance of the hyper server. Because of this, it has the potential to match
/// multiple patterns that you provide. It will call the first handler that it
/// matches against so the order in which you add routes matters.
pub struct Router {
    routes: RegexSet,
    patterns: Vec<Regex>,
    handlers: Vec<(Method, RouteHandler)>,
    not_found: RouteHandler,
}

impl Handler for Router {
    fn handle(&self, req: Request, res: Response) {
        let uri = format!("{}", req.uri);
        let matches = self.routes.matches(&uri);
        if !matches.matched_any() {
            (self.not_found)(req, res, None);
            return;
        }

        for index in matches {
            let (ref method, ref handler) = self.handlers[index];
            if method != &req.method {
                continue;
            }

            let ref regex = self.patterns[index];
            let captures = get_captures(regex, &uri);
            handler(req, res, captures);
            return;
        }

        not_allowed(req, res);
    }
}

pub struct RouterBuilder {
    routes: Vec<String>,
    handlers: Vec<(Method, RouteHandler)>,
    not_found: Option<RouteHandler>,
}

impl RouterBuilder {
    pub fn new() -> RouterBuilder {
        RouterBuilder {
            routes: vec![],
            handlers: vec![],
            not_found: None,
        }
    }

    pub fn route<H>(&mut self, verb: Method, route: &str, handler: H) -> &mut RouterBuilder where
        H: Fn(Request, Response, Captures) + Send + Sync + 'static
    {
        // Anchor the pattern at the start and end so routes only match exactly.
        let pattern = [r"\A", route, r"\z"].join("");

        self.routes.push(pattern);
        self.handlers.push((verb, Box::new(handler)));

        self
    }

    pub fn finalize(self) -> Result<Router, Error> {
        Ok(Router {
            routes: RegexSet::new(self.routes.iter())?,
            patterns: self.routes.iter().map(|route| Regex::new(route)).collect::<Result<_, _>>()?,
            handlers: self.handlers,
            not_found: self.not_found.unwrap_or_else(|| Box::new(default_not_found)),
        })
    }

    pub fn get<H>(&mut self, route: &str, handler: H) -> &mut RouterBuilder where 
        H: Fn(Request, Response, Captures) + Send + Sync + 'static
    {
        self.route(Method::Get, route, handler)
    }

    pub fn post<H>(&mut self, route: &str, handler: H) -> &mut RouterBuilder where 
        H: Fn(Request, Response, Captures) + Send + Sync + 'static
    {
        self.route(Method::Post, route, handler)
    }

    pub fn put<H>(&mut self, route: &str, handler: H) -> &mut RouterBuilder where 
        H: Fn(Request, Response, Captures) + Send + Sync + 'static
    {
        self.route(Method::Put, route, handler)
    }

    pub fn patch<H>(&mut self, route: &str, handler: H) -> &mut RouterBuilder where 
        H: Fn(Request, Response, Captures) + Send + Sync + 'static
    {
        self.route(Method::Patch, route, handler)
    }

    pub fn delete<H>(&mut self, route: &str, handler: H) -> &mut RouterBuilder where 
        H: Fn(Request, Response, Captures) + Send + Sync + 'static
    {
        self.route(Method::Delete, route, handler)
    }

    pub fn options<H>(&mut self, route: &str, handler: H) -> &mut RouterBuilder where 
        H: Fn(Request, Response, Captures) + Send + Sync + 'static
    {
        self.route(Method::Options, route, handler)
    }

    pub fn not_found<H>(&mut self, not_found: H) -> &mut RouterBuilder where
        H: Fn(Request, Response, Captures) + Send + Sync + 'static
    {
        self.not_found = Some(Box::new(not_found));
        self
    }
}

// The default 404 handler.
fn default_not_found(req: Request, mut res: Response, _: Captures) {
    let message = format!("No route handler found for {}", req.uri);
    *res.status_mut() = StatusCode::NotFound;
    res.send(message.as_bytes()).unwrap();
}

// This handler will get fired when a URI matches a route but contains the wrong method.
fn not_allowed(_: Request, mut res: Response) {
    *res.status_mut() = StatusCode::MethodNotAllowed;
    let res = res.start().unwrap();
    res.end().unwrap();
}

// Return that captures from a pattern that was matched.
fn get_captures(pattern: &Regex, uri: &str) -> Captures {
    // We know this compiles because it was part of the set.
    let caps = pattern.captures(uri);
    match caps {
        Some(caps) => {
            let mut v = vec![];
            for c in caps.iter() {
                v.push(c.unwrap().to_owned());
            }
            Some(v)
        }
        None => None,
    }
}

#[test]
fn bad_regular_expression() {
    fn test_handler(_: Request, _: Response, _: Captures) {}
    let mut router = RouterBuilder::new();
    router.route(Method::Get, r"/[", test_handler);
    let e = router.finalize();
    assert!(e.is_err());
}
