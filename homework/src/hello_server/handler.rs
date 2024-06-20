//! Request handler with a cache.

use std::io::prelude::*;
use std::net::TcpStream;
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::Duration;

use regex::bytes::Regex;

use super::cache::Cache;
use super::statistics::Report;

/// Computes the result for the given key. So expensive, much wow.
fn very_expensive_computation_that_takes_a_few_seconds(key: String) -> String {
    println!("[handler] doing computation for key: {key}");
    thread::sleep(Duration::from_secs(3));
    format!("{key}🐕")
}

/// Hello handler with a cache.
#[derive(Debug, Default, Clone)]
pub struct Handler {
    cache: Arc<Cache<String, String>>,
}

impl Handler {
    const OK: &'static str = "<!DOCTYPE html>
<html lang=\"en\">
  <head>
    <meta charset=\"utf-8\">
    <title>Hello!</title>
  </head>
  <body>
    <p>Result for key \"{key}\" is \"{result}\"</p>
  </body>
</html>";

    const NOT_FOUND: &'static str = "<!DOCTYPE html>
<html lang=\"en\">
  <head>
    <meta charset=\"utf-8\">
    <title>Hello!</title>
  </head>
  <body>
    <h1>Oops!</h1>
    <p>Sorry, I don't know what you're asking for.</p>
  </body>
</html>";

    /// Process the request and generate report.
    pub fn handle_conn(&self, request_id: usize, mut stream: TcpStream) -> Report {
        let mut buf = [0; 512];
        let _ = stream.read(&mut buf).unwrap();

        static REQUEST_REGEX: OnceLock<Regex> = OnceLock::<Regex>::new();

        let key = REQUEST_REGEX
            .get_or_init(|| Regex::new(r"GET /(?P<key>\w+) HTTP/1.1\r\n").unwrap())
            .captures(&buf)
            .and_then(|cap| cap.name("key"))
            .map(|key| String::from_utf8_lossy(key.as_bytes()));

        let resp = if let Some(ref key) = key {
            let result = self.cache.get_or_insert_with(
                key.to_string(),
                very_expensive_computation_that_takes_a_few_seconds,
            );
            format!(
                "HTTP/1.1 200 OK\r\n\r\n{}",
                Self::OK.replace("{key}", key).replace("{result}", &result)
            )
        } else {
            format!("HTTP/1.1 404 NOT FOUND\r\n\r\n{}", Self::NOT_FOUND)
        };

        stream.write_all(resp.as_bytes()).unwrap();

        Report::new(request_id, key.map(String::from))
    }
}
