use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};

/// The number of seconds in a year, which is the biggest `max-age` most caches accept.
const IMMUTABLE_MAX_AGE: u32 = 31536000;

/// The header values of the fixed directive combinations, so that the common cases need no allocation.
const WELL_KNOWN_HEADER_VALUES: [(CacheControl, &str); 4] = [
    (CacheControl::new(), ""),
    (CacheControl::no_cache(), "no-cache"),
    (CacheControl::no_store(), "no-store"),
    (CacheControl::immutable(), "public, max-age=31536000, immutable"),
];

/// Which caches are allowed to store a response.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum Cacheability {
    /// Send no `public`, `private` or `no-store` directive, so every cache falls back to its own default behavior.
    #[default]
    Unspecified,
    /// `public`: every cache may store the response, including CDNs and other shared caches.
    Public,
    /// `private`: only the browser that made the request may store the response.
    Private,
    /// `no-store`: no cache may store the response at all.
    NoStore,
}

/// The directives of a `Cache-Control` response header.
///
/// Every directive is a public field, so an uncommon combination can be built with the struct update syntax.
///
/// ```rust
/// use rocket_cache_response::CacheControl;
///
/// let cache_control = CacheControl {
///     s_max_age: Some(86400),
///     ..CacheControl::public(60)
/// };
///
/// assert_eq!("public, max-age=60, s-maxage=86400", cache_control.to_string());
/// ```
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct CacheControl {
    /// Which caches are allowed to store the response.
    pub cacheability:           Cacheability,
    /// `max-age`: for how many seconds the response stays fresh.
    pub max_age:                Option<u32>,
    /// `s-maxage`: replaces `max_age` for shared caches only.
    pub s_max_age:              Option<u32>,
    /// `no-cache`: a stored response must be revalidated with the origin server before every reuse.
    pub no_cache:               bool,
    /// `must-revalidate`: a stale response must not be reused without a successful revalidation.
    pub must_revalidate:        bool,
    /// `proxy-revalidate`: the same as `must_revalidate`, but it applies to shared caches only.
    pub proxy_revalidate:       bool,
    /// `no-transform`: caches and proxies must not modify the payload.
    pub no_transform:           bool,
    /// `immutable`: the body never changes, so the browser should not revalidate it even on a reload.
    pub immutable:              bool,
    /// `stale-while-revalidate`: for how many seconds a stale response may still be served while it is revalidated in the background.
    pub stale_while_revalidate: Option<u32>,
    /// `stale-if-error`: for how many seconds a stale response may still be served after the origin server fails.
    pub stale_if_error:         Option<u32>,
}

impl Default for CacheControl {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl CacheControl {
    /// Set no directive at all, which results in an empty header value.
    #[inline]
    pub const fn new() -> Self {
        Self {
            cacheability:           Cacheability::Unspecified,
            max_age:                None,
            s_max_age:              None,
            no_cache:               false,
            must_revalidate:        false,
            proxy_revalidate:       false,
            no_transform:           false,
            immutable:              false,
            stale_while_revalidate: None,
            stale_if_error:         None,
        }
    }

    /// `public, max-age=<max_age>`: every cache, shared ones included, may reuse the response for `max_age` seconds.
    #[inline]
    pub const fn public(max_age: u32) -> Self {
        let mut cache_control = Self::new();

        cache_control.cacheability = Cacheability::Public;
        cache_control.max_age = Some(max_age);

        cache_control
    }

    /// `private, max-age=<max_age>`: only the browser that made the request may reuse the response, for `max_age` seconds.
    #[inline]
    pub const fn private(max_age: u32) -> Self {
        let mut cache_control = Self::new();

        cache_control.cacheability = Cacheability::Private;
        cache_control.max_age = Some(max_age);

        cache_control
    }

    /// `no-cache`: a cache may store the response, but it must ask the origin server whether the stored copy is still good before every reuse.
    #[inline]
    pub const fn no_cache() -> Self {
        let mut cache_control = Self::new();

        cache_control.no_cache = true;

        cache_control
    }

    /// `no-store`: no cache may store the response at all.
    #[inline]
    pub const fn no_store() -> Self {
        let mut cache_control = Self::new();

        cache_control.cacheability = Cacheability::NoStore;

        cache_control
    }

    /// `public, max-age=31536000, immutable`: the response never changes, so it should be reused for a year without any revalidation.
    #[inline]
    pub const fn immutable() -> Self {
        let mut cache_control = Self::public(IMMUTABLE_MAX_AGE);

        cache_control.immutable = true;

        cache_control
    }

    /// Format the directives into a `Cache-Control` header value.
    #[inline]
    pub fn to_header_value(&self) -> Cow<'static, str> {
        for (cache_control, header_value) in WELL_KNOWN_HEADER_VALUES {
            if *self == cache_control {
                return Cow::Borrowed(header_value);
            }
        }

        Cow::Owned(self.to_string())
    }
}

impl Display for CacheControl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut separator = "";

        match self.cacheability {
            Cacheability::Unspecified => (),
            Cacheability::Public => write_directive(f, &mut separator, "public")?,
            Cacheability::Private => write_directive(f, &mut separator, "private")?,
            Cacheability::NoStore => write_directive(f, &mut separator, "no-store")?,
        }

        if self.no_cache {
            write_directive(f, &mut separator, "no-cache")?;
        }

        if let Some(max_age) = self.max_age {
            write_seconds_directive(f, &mut separator, "max-age", max_age)?;
        }

        if let Some(s_max_age) = self.s_max_age {
            write_seconds_directive(f, &mut separator, "s-maxage", s_max_age)?;
        }

        if self.must_revalidate {
            write_directive(f, &mut separator, "must-revalidate")?;
        }

        if self.proxy_revalidate {
            write_directive(f, &mut separator, "proxy-revalidate")?;
        }

        if self.no_transform {
            write_directive(f, &mut separator, "no-transform")?;
        }

        if self.immutable {
            write_directive(f, &mut separator, "immutable")?;
        }

        if let Some(stale_while_revalidate) = self.stale_while_revalidate {
            write_seconds_directive(
                f,
                &mut separator,
                "stale-while-revalidate",
                stale_while_revalidate,
            )?;
        }

        if let Some(stale_if_error) = self.stale_if_error {
            write_seconds_directive(f, &mut separator, "stale-if-error", stale_if_error)?;
        }

        Ok(())
    }
}

#[inline]
fn write_directive(
    f: &mut Formatter<'_>,
    separator: &mut &'static str,
    directive: &str,
) -> fmt::Result {
    f.write_str(separator)?;
    f.write_str(directive)?;

    *separator = ", ";

    Ok(())
}

#[inline]
fn write_seconds_directive(
    f: &mut Formatter<'_>,
    separator: &mut &'static str,
    directive: &str,
    seconds: u32,
) -> fmt::Result {
    write_directive(f, separator, directive)?;

    write!(f, "={seconds}")
}
