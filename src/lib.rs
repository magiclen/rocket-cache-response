/*!
# Cache Response for Rocket Framework

This crate provides a response struct used for HTTP cache control.

```rust
use rocket::get;
use rocket_cache_response::CacheResponse;

#[get("/")]
fn index() -> CacheResponse<&'static str> {
    CacheResponse::public("Hello world!", 3600)
}
```

Every directive is a public field of `CacheControl`, so an uncommon combination can be built with the struct update syntax.

```rust
use rocket::get;
use rocket_cache_response::{CacheControl, CacheResponse};

#[get("/")]
fn index() -> CacheResponse<&'static str> {
    CacheResponse::new("Hello world!", CacheControl {
        s_max_age:              Some(86400),
        stale_while_revalidate: Some(30),
        ..CacheControl::public(60)
    })
}
```

A browser holding on to an old response is a nuisance during development, so `only_release` drops the `Cache-Control` header when the program is built in the debug mode.

```rust
use rocket::get;
use rocket_cache_response::CacheResponse;

#[get("/")]
fn index() -> CacheResponse<&'static str> {
    CacheResponse::public("Hello world!", 3600).only_release()
}
```

## Which `Cache-Control` Should I Use?

| Situation | Directives | Shortcut |
| --------- | ---------- | -------- |
| A static file whose URL changes whenever its content changes, such as `app.9f2c1a.js` | `public, max-age=31536000, immutable` | `CacheResponse::immutable(responder)` |
| A public page or asset that only changes once in a while | `public, max-age=3600` | `CacheResponse::public(responder, 3600)` |
| A public response that a CDN should keep longer than a browser does | `public, max-age=60, s-maxage=86400` | `CacheControl { s_max_age: Some(86400), ..CacheControl::public(60) }` |
| A page or an API response that belongs to the logged-in user | `private, max-age=0` | `CacheResponse::private(responder, 0)` |
| A response that has to be checked with the server before every reuse | `no-cache` | `CacheResponse::no_cache(responder)` |
| Personal data, payment details or anything else sensitive | `no-store` | `CacheResponse::no_store(responder)` |

Things that are easy to get wrong:

* `no-cache` does not mean "do not cache". A cache may still store the response; it just has to ask the origin server whether the stored copy is still good before every reuse. Use `no-store` when the response must never be written to a cache at all.
* `no-cache` on its own still lets shared caches, such as a CDN or a company proxy, store the response. Add `private` whenever the body is meant for one user only.
* `must-revalidate` only takes effect after `max-age` has passed. It forbids a cache from serving the stale copy while the origin server is unreachable.
* `immutable` is honored even when the user presses the reload button, so only use it for URLs that get a new name whenever the content changes.
* `max-age` is counted by each cache on its own, so a CDN and a browser may hold their copies for different amounts of time. `s-maxage` is the way to give shared caches a lifetime of their own.
*/

pub extern crate rocket;

mod cache_control;

use rocket::{
    request::Request,
    response::{Responder, Response, Result},
};

pub use crate::cache_control::{CacheControl, Cacheability};

/// The responder with a `Cache-Control` header.
#[derive(Debug, Clone)]
pub struct CacheResponse<R> {
    /// The responder that produces the response itself.
    pub responder:     R,
    /// The directives to send. `None` adds no `Cache-Control` header at all.
    pub cache_control: Option<CacheControl>,
}

impl<R> CacheResponse<R> {
    /// Attach the given directives to a responder. Passing `None` adds no `Cache-Control` header at all.
    #[inline]
    pub fn new(responder: R, cache_control: impl Into<Option<CacheControl>>) -> Self {
        Self {
            responder,
            cache_control: cache_control.into(),
        }
    }

    /// `public, max-age=<max_age>`: every cache, shared ones included, may reuse the response for `max_age` seconds.
    #[inline]
    pub fn public(responder: R, max_age: u32) -> Self {
        Self::new(responder, CacheControl::public(max_age))
    }

    /// `private, max-age=<max_age>`: only the browser that made the request may reuse the response, for `max_age` seconds.
    #[inline]
    pub fn private(responder: R, max_age: u32) -> Self {
        Self::new(responder, CacheControl::private(max_age))
    }

    /// `no-cache`: a cache may store the response, but it must ask the origin server whether the stored copy is still good before every reuse.
    #[inline]
    pub fn no_cache(responder: R) -> Self {
        Self::new(responder, CacheControl::no_cache())
    }

    /// `no-store`: no cache may store the response at all.
    #[inline]
    pub fn no_store(responder: R) -> Self {
        Self::new(responder, CacheControl::no_store())
    }

    /// `public, max-age=31536000, immutable`: the response never changes, so it should be reused for a year without any revalidation.
    #[inline]
    pub fn immutable(responder: R) -> Self {
        Self::new(responder, CacheControl::immutable())
    }

    /// Drop the `Cache-Control` header when this program is built in the **debug** mode, so a browser always fetches the newest response during development.
    #[inline]
    pub fn only_release(mut self) -> Self {
        if cfg!(debug_assertions) {
            self.cache_control = None;
        }

        self
    }
}

impl<'r, 'o: 'r, R: Responder<'r, 'o>> Responder<'r, 'o> for CacheResponse<R> {
    fn respond_to(self, request: &'r Request<'_>) -> Result<'o> {
        let response = self.responder.respond_to(request)?;

        let header_value = match self.cache_control {
            Some(cache_control) => cache_control.to_header_value(),
            None => return Ok(response),
        };

        if header_value.is_empty() {
            // `CacheControl::new()` sets no directive, and an empty header value is worse than no header.
            return Ok(response);
        }

        Response::build_from(response).raw_header("Cache-Control", header_value).ok()
    }
}
