use rocket_cache_response::{CacheControl, Cacheability};

#[test]
fn constructors() {
    assert_eq!("", CacheControl::new().to_string());
    assert_eq!("public, max-age=3600", CacheControl::public(3600).to_string());
    assert_eq!("private, max-age=60", CacheControl::private(60).to_string());
    assert_eq!("no-cache", CacheControl::no_cache().to_string());
    assert_eq!("no-store", CacheControl::no_store().to_string());
    assert_eq!("public, max-age=31536000, immutable", CacheControl::immutable().to_string());
}

#[test]
fn struct_update() {
    assert_eq!(
        "public, max-age=60, s-maxage=86400, stale-while-revalidate=30",
        CacheControl {
            s_max_age: Some(86400),
            stale_while_revalidate: Some(30),
            ..CacheControl::public(60)
        }
        .to_string()
    );

    assert_eq!(
        "private, max-age=0, must-revalidate",
        CacheControl {
            must_revalidate: true,
            ..CacheControl::private(0)
        }
        .to_string()
    );
}

#[test]
fn every_directive() {
    assert_eq!(
        "public, no-cache, max-age=60, s-maxage=86400, must-revalidate, proxy-revalidate, \
         no-transform, immutable, stale-while-revalidate=30, stale-if-error=120",
        CacheControl {
            cacheability:           Cacheability::Public,
            max_age:                Some(60),
            s_max_age:              Some(86400),
            no_cache:               true,
            must_revalidate:        true,
            proxy_revalidate:       true,
            no_transform:           true,
            immutable:              true,
            stale_while_revalidate: Some(30),
            stale_if_error:         Some(120),
        }
        .to_string()
    );
}

/// The borrowed fast path has its header values written out, so it has to agree with `Display`.
#[test]
fn to_header_value() {
    for cache_control in [
        CacheControl::new(),
        CacheControl::public(3600),
        CacheControl::private(60),
        CacheControl::no_cache(),
        CacheControl::no_store(),
        CacheControl::immutable(),
    ] {
        assert_eq!(cache_control.to_string(), cache_control.to_header_value());
    }
}
