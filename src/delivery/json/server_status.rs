//
// Copyright:: Copyright (c) 2017 Chef Software, Inc.
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

// Collection of structs representing the _status endpoint json.
#[derive(Serialize, Deserialize)]
pub struct ServerStatus {
    pub configuration_mode: String,
    pub status: String,
    pub upstreams: Vec<Upstreams>,
    pub fips_mode: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct Upstreams {
    pub lsyncd: Lsyncd,
    pub postgres: Postgres,
    pub rabbitmq: Rabbitmq,
}

#[derive(Serialize, Deserialize)]
pub struct Lsyncd {
    pub status: String,
}

#[derive(Serialize, Deserialize)]
pub struct Postgres {
    pub status: String,
}

#[derive(Serialize, Deserialize)]
pub struct Rabbitmq {
    pub status: String,
    pub node_health: Option<NodeHealth>,
    pub vhost_aliveness: Option<VhostAliveness>,
}

#[derive(Serialize, Deserialize)]
pub struct NodeHealth {
    pub status: String,
}

#[derive(Serialize, Deserialize)]
pub struct VhostAliveness {
    pub status: String,
}
