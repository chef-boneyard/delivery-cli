use hyper::header::{Header, HeaderFormat};
use std::fmt;
use hyper::header::parsing::from_one_raw_str;

/// Create a hyper header type for an HTTP header with name $h_string
/// whose value is a string.
macro_rules! hyper_header {
    ($h_name:ident, $h_string:expr) => (
        #[derive(Clone, PartialEq, Debug)]
        pub struct $h_name(pub String);

        impl Header for $h_name {
            fn header_name() -> &'static str {
                $h_string
            }

            fn parse_header(raw: &[Vec<u8>]) -> Option<$h_name> {
                from_one_raw_str(raw).map(|s| $h_name(s))
            }
        }

        impl HeaderFormat for $h_name {
            fn fmt_header(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str(&*self.0)
            }
        })
}

hyper_header!(ChefDeliveryToken, "chef-delivery-token");
hyper_header!(ChefDeliveryUser,  "chef-delivery-user");
