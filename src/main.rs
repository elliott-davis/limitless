extern crate frank_jwt;
#[macro_use]
extern crate serde_json;
extern crate clap;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;

use clap::{App, Arg};
use serde_json::Value;
use frank_jwt::{Algorithm, encode};
use futures::{Future, Stream};
use hyper::{Body, Client, Method, Request};
use hyper::header::{Accept, Authorization, Bearer, qitem, UserAgent};
use hyper_tls::HttpsConnector;

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_core::reactor::Core;

const USER_AGENT: &'static str = "Habitat Rate Limit Checker";

fn main() {
    let matches = app().get_matches();

    let key = matches.value_of("key_file").expect("Key file requred");
    let app_id = matches.value_of("app_id").expect("App ID required");
    let install_id = matches.value_of("installation_id").expect("Installation ID required");

    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(1, &handle).unwrap())
        .build(&handle);
    let uri = format!("https://api.github.com/installations/{}/access_tokens", install_id).parse().unwrap();
    let mut req = Request::new(Method::Post, uri);
    req.headers_mut().set(Authorization(
       Bearer {
           token: format!("{}", jwt(key, app_id).expect("Bad JWT"))
       }
    ));
    req.headers_mut().set(UserAgent::new(USER_AGENT));
    req.headers_mut().set(Accept(vec![
        qitem("application/vnd.github.machine-man-preview+json".parse().unwrap())]));
    let post = client.request(req).and_then(|res| {
        res.body().concat2().and_then(move |body| {
            let v: Value = serde_json::from_slice(&body).expect("Failed to get Bearer token");
            Ok(v)
        })
    }).and_then(|val| {
        let uri2 = "https://api.github.com/rate_limit".parse().unwrap();
        let mut req2: Request<Body> = Request::new(Method::Get, uri2);
        req2.headers_mut().set(Authorization(
            Bearer {
                token: val["token"].as_str().expect("Failed to parse token").to_string()
            }
        ));
        req2.headers_mut().set(UserAgent::new(USER_AGENT));
        client.request(req2).and_then(move |res| {
            res.body().concat2().and_then(move |body| {
                let v: Value = serde_json::from_slice(&body).expect("Failed to fetch rate limit stats");
                Ok(v)
            })
        })
    });

    let token = core.run(post).unwrap();
    println!("{}", serde_json::to_string_pretty(&token).expect("Failed to convert JSON to string"));
}

fn jwt(key: &str, app_id: &str) -> Result<String, frank_jwt::Error> {
    let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let header = json!({});
    let payload = json!({
        "iat": since_the_epoch.as_secs(),
        "exp": (since_the_epoch.as_secs() + (60 * 10)),
        "iss": app_id,
    });
    encode(header, &PathBuf::from(key), &payload, Algorithm::RS256)
}

fn app<'a, 'b>() -> App<'a, 'b> {
    App::new("Limitless")
        .version("0.1")
        .author("Elliott Davis <elliott@excellent.io>")
        .about("Returns the GitHub ratelimit payload for your app")
        .arg(Arg::with_name("key_file")
                    .short("k")
                    .long("key")
                    .value_name("KEY_FILE")
                    .help("GitHub pem key for your GitHub app")
                    .required(true)
                    .takes_value(true))
        .arg(Arg::with_name("app_id")
                    .short("a")
                    .long("app_id")
                    .value_name("APP_ID")
                    .help("GitHub application ID")
                    .required(true)
                    .takes_value(true))
        .arg(Arg::with_name("installation_id")
                    .short("i")
                    .long("installation_id")
                    .value_name("install_id")
                    .help("GitHub installation ID")
                    .required(true)
                    .takes_value(true))
}