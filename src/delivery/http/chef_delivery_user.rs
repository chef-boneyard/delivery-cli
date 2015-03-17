//
// Copyright:: Copyright (c) 2015 Chef Software, Inc.
// License:: Apache License, Version 2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

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
