extern crate hyper;
extern crate regex;

use std::collections::HashMap;

use hyper::server::{Handler, Request, Response};
use regex::RegexSet;

pub type RouterFn = fn(Request, Response);

const MIN_ROUTES: usize = 2;

pub struct Router {
    not_found: Option<RouterFn>,
    routes: RegexSet,
    route_list: Vec<String>,
    route_map: HashMap<String, RouterFn>
}

impl Handler for Router {
    // The handle method for the router simply tries to match the URI against
    // the first pattern that it can which in turn calls its associated handle
    // function passing the hyper Request and Response structures.
    fn handle(&self, req: Request, res: Response) {
        let uri = format!("{}", req.uri);
        let matches = self.routes.matches(&uri);
        let route = matches.iter().next();
        match route {
            Some(r) => {
                let key = &self.route_list[r];
                let handler = self.route_map.get(key).unwrap();
                handler(req, res);
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
            routes: RegexSet::new(&["", ""]).unwrap(),
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
    pub fn finalize(&mut self) -> Result<(), String> {
        if self.route_list.len() < MIN_ROUTES {
            return Err("The router must contain 2 or more routes".to_owned());
        }

        // Check if the user added a 404 handler, else use the default.
        match self.not_found {
            Some(_) => {},
            None => { self.not_found = Some(default_not_found); }
        }

        let re_routes = RegexSet::new(self.route_list.iter());
        match re_routes {
            Ok(r) => {
                self.routes = r;
                Ok(())
            }
            Err(e) => {
                Err(format!("Error making RegexSet: {}", e))
            }
        }
    }

    /// Add a function to handle routes that get no matches.
    pub fn add_not_found(&mut self, not_found: RouterFn) {
        self.not_found = Some(not_found)
    }
}

fn default_not_found(req: Request, res: Response) {
    let message = format!("No route handler found for {}", req.uri);
    res.send(message.as_bytes()).unwrap();
}

fn test_handler(_: Request, _: Response) {}

#[test]
#[should_panic]
fn less_than_two_routes() {
    let mut router = Router::new();
    router.add_route("/", test_handler);
    router.finalize().unwrap();
}
