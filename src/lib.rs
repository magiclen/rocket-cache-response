//! # Cache Response for Rocket Framework
//! This crate provides a response struct used for HTTP cache control.

extern crate rocket;

use rocket::response::{self, Response, Responder};
use rocket::request::Request;

pub enum CacheControl<'a, R: Responder<'a>> {
    Public(R, u32, bool),
    Private(R, u32),
    NoCache(R),
    NoStore(R),
    NoCacheControl(R, &'a str), // this str is a trick to prevent unused lifetime parameter 'a
}

impl<'a, R: Responder<'a>> Responder<'a> for CacheControl<'a, R> {
    fn respond_to(self, req: &Request) -> response::Result<'a> {
        return match self {
            CacheControl::Public(responder, max_age, must_revalidate) => {
                Response::build_from(responder.respond_to(req)?)
                    .raw_header("Cache-Control", if must_revalidate { format!("must-revalidate, public, max-age={}", max_age) } else { format!("public, max-age={}", max_age) })
                    .ok()
            }
            CacheControl::Private(responder, max_age) => {
                Response::build_from(responder.respond_to(req)?)
                    .raw_header("Cache-Control", format!("private, max-age={}", max_age))
                    .ok()
            }
            CacheControl::NoCache(responder) => {
                Response::build_from(responder.respond_to(req)?)
                    .raw_header("Cache-Control", "no-cache")
                    .ok()
            }
            CacheControl::NoStore(responder) => {
                Response::build_from(responder.respond_to(req)?)
                    .raw_header("Cache-Control", "no-store")
                    .ok()
            }
            CacheControl::NoCacheControl(responder, _) => {
                Response::build_from(responder.respond_to(req)?)
                    .ok()
            }
        };
    }
}

impl<'a, R: Responder<'a>> CacheControl<'a, R> {
    pub fn create_public_cache(responder: R, max_age: u32, must_revalidate: bool) -> CacheControl<'a, R> {
        CacheControl::Public(
            responder,
            max_age,
            must_revalidate,
        )
    }

    pub fn create_private_cache(responder: R, max_age: u32) -> CacheControl<'a, R> {
        CacheControl::Private(
            responder,
            max_age,
        )
    }

    pub fn create_no_cache(responder: R) -> CacheControl<'a, R> {
        CacheControl::NoCache(responder)
    }

    pub fn create_no_store(responder: R) -> CacheControl<'a, R> {
        CacheControl::NoStore(responder)
    }

    pub fn create_no_cache_control(responder: R) -> CacheControl<'a, R> {
        CacheControl::NoCacheControl(responder, "")
    }
}