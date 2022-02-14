use std::{str::FromStr};

use nickel::{
  hyper::{
    header::ContentType,
    method::Method::{Get, Head},
    mime::{Mime},
  },
  status::StatusCode::NotFound,
  HttpRouter, Middleware, MiddlewareResult, Nickel, Request, Response,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "examples/public/"]
#[prefix = "static"]
struct Asset;

impl<D> Middleware<D> for Asset {
  fn invoke<'a>(&self, req: &mut Request<D>, res: Response<'a, D>) -> MiddlewareResult<'a, D> {
    match req.origin.method {
      Get | Head => {
        if let Some(path) = extract_path(req) {
          send_file(path, res)
        } else {
          res.error(NotFound, "Not found")
        }
      }
      _ => res.next_middleware(),
    }
  }
}

fn send_file<'a, D>(path: &str, mut res: Response<'a, D>) -> MiddlewareResult<'a, D> {
  if let Some(file) = <Asset as RustEmbed>::get(path) {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    // Lookup mime from file extension
    res.set(ContentType(Mime::from_str(mime.as_ref()).unwrap()));
    res.send(file.data.as_ref())
  } else {
    res.error(NotFound, "Not found")
  }
}

fn extract_path<'a, D>(req: &'a mut Request<D>) -> Option<&'a str> {
  req.path_without_query().map(|path| match path {
    "/" => "index.html",
    path => &path[1..],
  })
}

fn main() {
  let mut server = Nickel::new();

  server.get("/static", Asset);

  server.listen("127.0.0.1:8080").unwrap();
}
