/*!
# Cache Response for Rocket Framework

This crate provides a response struct used for HTTP cache control.

See `examples`.
*/

extern crate rocket;

use std::marker::PhantomData;

use rocket::request::Request;
use rocket::response::{Responder, Response, Result};

/// The responder with a `Cache-Control` header.
pub type CacheResponse<R> = CacheResponsePro<'static, 'static, R>;

/// The responder with a `Cache-Control` header.
#[derive(Debug)]
pub enum CacheResponsePro<'r, 'o: 'r, R: Responder<'r, 'o>> {
    Public {
        responder: R,
        max_age: u32,
        must_revalidate: bool,
    },
    Private {
        responder: R,
        max_age: u32,
    },
    NoCache(R),
    NoStore(R),
    NoCacheControl(R),
    #[doc(hidden)]
    _Phantom(PhantomData<(&'r R, &'o R)>),
}

#[rocket::async_trait]
impl<'r, 'o: 'r, R: Responder<'r, 'o>> Responder<'r, 'o> for CacheResponsePro<'r, 'o, R> {
    fn respond_to(self, req: &'r Request<'_>) -> Result<'o> {
        match self {
            CacheResponsePro::Public {
                responder,
                max_age,
                must_revalidate,
            } => {
                Response::build_from(responder.respond_to(req)?)
                    .raw_header(
                        "Cache-Control",
                        if must_revalidate {
                            format!("must-revalidate, public, max-age={}", max_age)
                        } else {
                            format!("public, max-age={}", max_age)
                        },
                    )
                    .ok()
            }
            CacheResponsePro::Private {
                responder,
                max_age,
            } => {
                Response::build_from(responder.respond_to(req)?)
                    .raw_header("Cache-Control", format!("private, max-age={}", max_age))
                    .ok()
            }
            CacheResponsePro::NoCache(responder) => {
                Response::build_from(responder.respond_to(req)?)
                    .raw_header("Cache-Control", "no-cache")
                    .ok()
            }
            CacheResponsePro::NoStore(responder) => {
                Response::build_from(responder.respond_to(req)?)
                    .raw_header("Cache-Control", "no-store")
                    .ok()
            }
            CacheResponsePro::NoCacheControl(responder) => {
                Response::build_from(responder.respond_to(req)?).ok()
            }
            _ => unimplemented!(),
        }
    }
}

impl<'r, 'o: 'r, R: Responder<'r, 'o>> CacheResponsePro<'r, 'o, R> {
    /// Use public cache only when this program is built on the **release** mode.
    #[cfg(debug_assertions)]
    pub fn public_only_release(
        responder: R,
        _max_age: u32,
        _must_revalidate: bool,
    ) -> CacheResponsePro<'r, 'o, R> {
        CacheResponsePro::NoCacheControl(responder)
    }

    /// Use public cache only when this program is built on the **release** mode.
    #[cfg(not(debug_assertions))]
    pub fn public_only_release(
        responder: R,
        max_age: u32,
        must_revalidate: bool,
    ) -> CacheResponsePro<'r, 'o, R> {
        CacheResponsePro::Public {
            responder,
            max_age,
            must_revalidate,
        }
    }

    /// Use private cache only when this program is built on the **release** mode.
    #[cfg(debug_assertions)]
    pub fn private_only_release(responder: R, _max_age: u32) -> CacheResponsePro<'r, 'o, R> {
        CacheResponsePro::NoCacheControl(responder)
    }

    /// Use private cache only when this program is built on the **release** mode.
    #[cfg(not(debug_assertions))]
    pub fn private_only_release(responder: R, max_age: u32) -> CacheResponsePro<'r, 'o, R> {
        CacheResponsePro::Private {
            responder,
            max_age,
        }
    }
}
