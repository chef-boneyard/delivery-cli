use hyper::header::{Header, HeaderFormat};
use std::fmt;
use hyper::header::prasing::from_one_raw_str;

/// The `Chef-Delivery-User` header field.
///
/// They can contain any value, so it just wraps a `String`.
#[derive(Clone, PartialEq, Show, Debug)]
pub struct ChefDeliveryUser(pub String);

impl Header for ChefDeliveryUser {
    fn header_name(_: Option<ChefDeliveryUser>) -> &'static str {
        "User-Agent"
    }

    fn parse_header(raw: &[Vec<u8>]) -> Option<ChefDeliveryUser> {
        from_one_raw_str(raw).map(|s| ChefDeliveryUser(s))
    }
}

impl HeaderFormat for ChefDeliveryUser {
    fn fmt_header(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&*self.0)
    }
}
