extern crate hyper;
extern crate regex;

use std::collections::HashMap;

use hyper::method::Method;
use hyper::server::{Handler, Request, Response};
use hyper::status::StatusCode;
use regex::{Regex, RegexSet};

use error::RouterError;

pub type Captures = Option<Vec<String>>;
pub type NotFoundFn = fn(Request, Response);

#[derive(PartialEq, Eq, Hash)]
pub struct RouteInfo {
    route: String,
    verb: Method
}

/// The Router struct contains the information for your app to route requests
/// properly based on their HTTP method and matching route. It allows the use
/// of a custom 404 handler if desired but provides a default as well.
///
/// Under the hood a Router uses a RegexSet to match URI's that come in to the
/// instance of the hyper server. Because of this, it has the potential to match
/// multiple patterns that you provide. It will call the first handler that it
/// matches against so the order in which you add routes matters.
pub struct Router {
    /// A custom 404 handler that you can provide.
    pub not_found: Option<NotFoundFn>,

    routes: Option<RegexSet>,
    route_list: Vec<String>,
    compiled_list: Vec<Regex>,
    route_map: HashMap<RouteInfo, Box<Fn(Request, Response, Captures)>>
}

// These are needed to satisfy the Handler trait while using Fn to accept closures.
unsafe impl Send for Router { }
unsafe impl Sync for Router { }

impl Handler for Router {
    // The handle method for the router simply tries to match the URI against
    // the first pattern that it can which in turn calls its associated handle
    // function passing the hyper Request and Response structures.
    fn handle(&self, req: Request, res: Response) {
        let uri = format!("{}", req.uri);
        let matches = self.routes.as_ref().unwrap().matches(&uri);
        let index = matches.iter().next();
        match index {
            Some(i) => {
                let route = &self.route_list[i];
                let route_info = RouteInfo{route: route.clone(), verb: req.method.clone()};
                let handler = self.route_map.get(&route_info);
                match handler {
                    Some(h) => {
                        let compiled_pattern = &self.compiled_list[i];
                        let captures = get_captures(compiled_pattern, &uri);
                        h(req, res, captures);
                    }
                    None => {
                        not_allowed(req, res);
                    }
                }
            },
            None => self.not_found.unwrap()(req, res)
        }
    }
}

impl Router {
    /// Construct a new Router to maintain the routes and their handler
    /// functions.
    pub fn new() -> Router {
        Router {
            not_found: None,
            routes: None,
            route_list: Vec::new(),
            compiled_list: Vec::new(),
            route_map: HashMap::new(),
        }
    }

    /// Add a route to the router and give it a function to call when the route
    /// is matched against. You can call this explicitly or use the convenience
    /// methods defined below.
    pub fn add_route<H>(&mut self, verb: Method, route: &str, handler: H)
        where H: Fn(Request, Response, Captures) + Send + Sync + 'static {
        let route_info = RouteInfo{route: route.to_owned(), verb: verb};
        let pattern = Regex::new(route);
        match pattern {
            Ok(p) => { self.compiled_list.push(p); },
            Err(e) => { println!("Not adding this route due to error: {}", e); }
        }
        self.route_list.push(route.to_owned());
        self.route_map.insert(route_info, Box::new(handler));
    }


    /// A convenience method for OPTIONS requests.
    pub fn options<H>(&mut self, route: &str, handler: H)
        where H: Fn(Request, Response, Captures) + Send + Sync + 'static {
        self.add_route(Method::Options, route, handler);
    }

    /// A convenience method for GET requests.
    pub fn get<H>(&mut self, route: &str, handler: H)
        where H: Fn(Request, Response, Captures) + Send + Sync + 'static {
        self.add_route(Method::Get, route, handler);
    }

    /// A convenience method for POST requests.
    pub fn post<H>(&mut self, route: &str, handler: H)
        where H: Fn(Request, Response, Captures) + Send + Sync + 'static {
        self.add_route(Method::Post, route, handler);
    }

    /// A convenience method for PUT requests.
    pub fn put<H>(&mut self, route: &str, handler: H)
        where H: Fn(Request, Response, Captures) + Send + Sync + 'static {
        self.add_route(Method::Put, route, handler);
    }

    /// A convenience method for DELETE requests.
    pub fn delete<H>(&mut self, route: &str, handler: H)
        where H: Fn(Request, Response, Captures) + Send + Sync + 'static {
        self.add_route(Method::Delete, route, handler);
    }

    /// A convenience method for PATCH requests.
    pub fn patch<H>(&mut self, route: &str, handler: H)
        where H: Fn(Request, Response, Captures) + Send + Sync + 'static {
        self.add_route(Method::Patch, route, handler);
    }

    /// This function ensures that a valid `RegexSet` could be made from the route
    /// vector that was built while using the functions that add routes. It
    /// requires that there exist at least one route so that the RegexSet can be
    /// successfully constructed.
    ///
    /// It will also ensure that there is a handler for routes that do not match
    /// any available in the set.
    pub fn finalize(&mut self) -> Result<(), RouterError> {
        if self.route_list.len() == 0  {
            return Err(RouterError::TooFewRoutes);
        }

        // Check if the user added a 404 handler, else use the default.
        match self.not_found {
            Some(_) => {},
            None => { self.not_found = Some(default_not_found); }
        }

        let re_routes = RegexSet::new(self.route_list.iter());
        match re_routes {
            Ok(r) => {
                self.routes = Some(r);
                Ok(())
            }
            Err(_) => {
                Err(RouterError::BadSet)
            }
        }
    }

    /// Add a function to handle routes that get no matches.
    pub fn add_not_found(&mut self, not_found: NotFoundFn) {
        self.not_found = Some(not_found)
    }
}

// The default 404 handler.
fn default_not_found(req: Request, mut res: Response) {
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
        },
        None => None
    }
}

mod error;

#[test]
fn no_routes() {
    let mut router = Router::new();
    let e = router.finalize();
    assert_eq!(e.err(), Some(RouterError::TooFewRoutes));
}

#[test]
fn bad_regular_expression() {
    fn test_handler(_: Request, _: Response, _: Captures) {}
    let mut router = Router::new();
    router.add_route(Method::Get, r"/[", test_handler);
    let e = router.finalize();
    assert_eq!(e.err(), Some(RouterError::BadSet));
}
