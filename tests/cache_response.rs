use rocket::{get, local::blocking::Client, routes};
use rocket_cache_response::{CacheControl, CacheResponse};

#[get("/public")]
fn public_page() -> CacheResponse<&'static str> {
    CacheResponse::public("public", 3600)
}

#[get("/no-store")]
fn no_store_page() -> CacheResponse<&'static str> {
    CacheResponse::no_store("no-store")
}

#[get("/cdn")]
fn cdn_page() -> CacheResponse<&'static str> {
    CacheResponse::new("cdn", CacheControl {
        s_max_age: Some(86400),
        ..CacheControl::public(60)
    })
}

#[get("/no-header")]
fn no_header_page() -> CacheResponse<&'static str> {
    CacheResponse::new("no-header", None)
}

#[test]
fn cache_control_header() {
    let client = Client::tracked(rocket::build().mount("/", routes![
        public_page,
        no_store_page,
        cdn_page,
        no_header_page
    ]))
    .unwrap();

    assert_eq!(
        Some("public, max-age=3600"),
        client.get("/public").dispatch().headers().get_one("Cache-Control")
    );

    assert_eq!(
        Some("no-store"),
        client.get("/no-store").dispatch().headers().get_one("Cache-Control")
    );

    assert_eq!(
        Some("public, max-age=60, s-maxage=86400"),
        client.get("/cdn").dispatch().headers().get_one("Cache-Control")
    );

    assert_eq!(None, client.get("/no-header").dispatch().headers().get_one("Cache-Control"));
}
