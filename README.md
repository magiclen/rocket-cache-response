Cache Response for Rocket Framework
====================

[![CI](https://github.com/magiclen/rocket-cache-response/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/rocket-cache-response/actions/workflows/ci.yml)

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

## Crates.io

https://crates.io/crates/rocket-cache-response

## Documentation

https://docs.rs/rocket-cache-response

## License

[MIT](LICENSE)
