#[macro_use]
extern crate rocket;

extern crate rocket_cache_response;

extern crate chrono;

use rocket_cache_response::CacheResponse;

use chrono::prelude::*;

#[get("/")]
fn index() -> CacheResponse<String> {
    CacheResponse::Public {
        responder: format!("Current Time: {}\n\nTry to re-open this page repeatedly without pressing the refresh(F5) or forced-refresh(Ctrl+F5) buttons.", Utc::now().to_rfc3339()),
        max_age: 10, // cached for seconds
        must_revalidate: false,
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
