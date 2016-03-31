extern crate hyper;
extern crate regex;

use std::collections::HashMap;

use hyper::server::{Handler, Request, Response};
use regex::{Regex, RegexSet};

use error::RouterError;

pub type Captures = Option<Vec<String>>;
pub type RouterFn = fn(Request, Response, Captures);

pub struct Router {
    not_found: Option<RouterFn>,
    routes: Option<RegexSet>,
    route_list: Vec<String>,
    route_map: HashMap<String, RouterFn>
}

impl Handler for Router {
    // The handle method for the router simply tries to match the URI against
    // the first pattern that it can which in turn calls its associated handle
    // function passing the hyper Request and Response structures.
    fn handle(&self, req: Request, res: Response) {
        let uri = format!("{}", req.uri);
        let matches = self.routes.clone().unwrap().matches(&uri);
        let route = matches.iter().next();
        match route {
            Some(r) => {
                let key = &self.route_list[r];
                let captures = get_captures(key, &uri);
                let handler = self.route_map.get(key).unwrap();
                handler(req, res, captures);
            },
            // There is no point in passing captures to a route handler that
            // wasn't found.
            None => self.not_found.unwrap()(req, res, None)
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
            route_map: HashMap::new(),
        }
    }

    /// Add a route to the router and give it a function to call when the route
    /// is matched against.
    pub fn add_route(&mut self, route: &str, handler: RouterFn) {
        self.route_list.push(route.to_owned());
        self.route_map.insert(route.to_owned(), handler);
    }

    /// This function ensures that a valid RegexSet could be made from the route
    /// vector that was built while using the `add_route` function. It also
    /// requires that there exist two or more routes so that the RegexSet can be
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
    pub fn add_not_found(&mut self, not_found: RouterFn) {
        self.not_found = Some(not_found)
    }
}

fn default_not_found(req: Request, res: Response, _: Captures) {
    let message = format!("No route handler found for {}", req.uri);
    res.send(message.as_bytes()).unwrap();
}

fn get_captures(route: &str, uri: &str) -> Captures {
    // We know this compiles because it was part of the set.
    let re = Regex::new(route).unwrap();
    let caps = re.captures(uri);
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
fn less_than_two_routes() {
    let mut router = Router::new();
    let e = router.finalize();
    assert_eq!(e.err(), Some(RouterError::TooFewRoutes));
}

#[test]
fn bad_regular_expression() {
    fn test_handler(_: Request, _: Response, _: Captures) {}
    let mut router = Router::new();
    router.add_route(r"/[", test_handler);
    let e = router.finalize();
    assert_eq!(e.err(), Some(RouterError::BadSet));
}
