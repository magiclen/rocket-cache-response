/*!
# Cache Response for Rocket Framework

This crate provides a response struct used for HTTP cache control.

See `examples`.
*/

extern crate rocket;

use std::marker::PhantomData;

use rocket::http::hyper::header::{CacheControl, CacheDirective};
use rocket::request::Request;
use rocket::response::{Responder, Response, Result};

/// The responder with a `Cache-Control` header.
pub type CacheResponse<R> = CacheResponsePro<'static, R>;

/// The responder with a `Cache-Control` header.
#[derive(Debug)]
pub enum CacheResponsePro<'r, R: Responder<'r>> {
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
    _Phantom(PhantomData<&'r R>),
}

impl<'r, R: Responder<'r>> Responder<'r> for CacheResponsePro<'r, R> {
    fn respond_to(self, req: &Request) -> Result<'r> {
        match self {
            CacheResponsePro::Public {
                responder,
                max_age,
                must_revalidate,
            } => {
                let cache_control = CacheControl({
                    let mut v = vec![CacheDirective::Public, CacheDirective::MaxAge(max_age)];

                    if must_revalidate {
                        v.push(CacheDirective::MustRevalidate);
                    }

                    v
                });

                Response::build_from(responder.respond_to(req)?).header(cache_control).ok()
            }
            CacheResponsePro::Private {
                responder,
                max_age,
            } => {
                Response::build_from(responder.respond_to(req)?)
                    .header(CacheControl(vec![
                        CacheDirective::Private,
                        CacheDirective::MaxAge(max_age),
                    ]))
                    .ok()
            }
            CacheResponsePro::NoCache(responder) => {
                Response::build_from(responder.respond_to(req)?)
                    .header(CacheControl(vec![CacheDirective::NoCache]))
                    .ok()
            }
            CacheResponsePro::NoStore(responder) => {
                Response::build_from(responder.respond_to(req)?)
                    .header(CacheControl(vec![CacheDirective::NoStore]))
                    .ok()
            }
            CacheResponsePro::NoCacheControl(responder) => {
                Response::build_from(responder.respond_to(req)?).ok()
            }
            _ => unimplemented!(),
        }
    }
}

impl<'r, R: Responder<'r>> CacheResponsePro<'r, R> {
    /// Use public cache only when this program is built on the **release** mode.
    #[cfg(debug_assertions)]
    pub fn public_only_release(
        responder: R,
        _max_age: u32,
        _must_revalidate: bool,
    ) -> CacheResponsePro<'r, R> {
        CacheResponsePro::NoCacheControl(responder)
    }

    /// Use public cache only when this program is built on the **release** mode.
    #[cfg(not(debug_assertions))]
    pub fn public_only_release(
        responder: R,
        max_age: u32,
        must_revalidate: bool,
    ) -> CacheResponsePro<'r, R> {
        CacheResponsePro::Public {
            responder,
            max_age,
            must_revalidate,
        }
    }

    /// Use private cache only when this program is built on the **release** mode.
    #[cfg(debug_assertions)]
    pub fn private_only_release(responder: R, _max_age: u32) -> CacheResponsePro<'r, R> {
        CacheResponsePro::NoCacheControl(responder)
    }

    /// Use private cache only when this program is built on the **release** mode.
    #[cfg(not(debug_assertions))]
    pub fn private_only_release(responder: R, max_age: u32) -> CacheResponsePro<'r, R> {
        CacheResponsePro::Private {
            responder,
            max_age,
        }
    }
}
