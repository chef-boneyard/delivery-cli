# Delivery CLI

The CLI for Chef Delivery. Written in Rust, super experimental, will probably hurt your kittens.

_This is alpha stage software, and is in a state of perpetual change. Use at your own risk!_

## Usage

Start using `delivery` by issuing the setup command:

```shell
$ delivery setup --user USER --server SERVER --ent ENTERPRISE --org ORGANIZATION --config-path /Users/adam
```

This will configure delivery to, by default, contact the delivery server at SERVER, with a default
ENTERPRISE and ORGANIZATION.

### Job

The Delivery CLI is going to also encompass the act of seting up a workspace,
configuring it to run, and then actually running a delivery job. The goal is:

#### To be able to run any delivery phase from the command line, as if your laptop was a build node.

As a developer, it's good best practice to verify that your code will work
locally before submitting it. What if we could validate that with the identical
behavior we would have on a build node in our pipeline?

You can run:

```bash
$ delivery job verify unit
Chef Delivery
Loading configuration from /Users/adam/src/opscode/delivery/opscode/delivery-cli
Starting job for verify unit
Creating workspace
Cloning repository, and merging adam/job to master
Configuring the job
Running the job
Starting Chef Client, version 11.18.0.rc.1
resolving cookbooks for run list: ["delivery_rust::unit"]
Synchronizing Cookbooks:
  - delivery_rust
  - build-essential
Compiling Cookbooks...
Converging 2 resources
Recipe: delivery_rust::unit
  * execute[cargo clean] action run
    - execute cargo clean
  * execute[cargo test] action run
    - execute cargo test

Running handlers:
Running handlers complete
Chef Client finished, 2/2 resources updated in 32.770955 seconds
```

Which will keep a persistent, local cache, and behave as a build node would.

This also has a delightful side effect, which is that anyone can use the delivery
cli to get the *job* behaviors of delivery, including integrating them in to existing
legacy solutions.

2) Make the setup and execution of the build job straightforward and easy
   to debug.

First we create the workspace directories, then we clone the project we are to
build, configure the Chef environent, and execute the job.

To setup a job, the delivery cli reads the `.delivery/config.json` file, and
looks for its `build_cookbook` parameter. It takes 3 forms:

#### From a local directory

```json
{
    "version": "1",
    "build_cookbook": {
      "name": "delivery_rust",
      "path": "cookbooks/delivery_rust"
    },
    "build_nodes": {
        "default"    : ["name:delivery-builder*"]
    }
}
```

#### From a Git source

```json
{
    "version": "1",
    "build_cookbook": {
      "name": "delivery_rust",
      "git": "ssh://..."
    },
    "build_nodes": {
        "default"    : ["name:delivery-builder*"]
    }
}
```

#### From a Supermarket

```json
{
    "version": "1",
    "build_cookbook": {
      "name": "delivery_rust",
      "supermarket": "https://supermarket.chef.io"
    },
    "build_nodes": {
        "default"    : ["name:delivery-builder*"]
    }
}
```

It will then retrieve the source, and execute a `berks vendor` on it. This fetches
any dependencies it may have. We then execute:

```bash
$ chef-client -z -j ../chef/dna.json -c ../chef/config.rb -r 'delivery_rust::unit'
```

3) Optimize things like the idempotence check for build node setup.

When dispatching the job, we check to see if we have executed the build node setup
recipe. We do the following:

* Check to see if we have run it before - if we haven't, execute it.
* If we have run it in the last 24 hours, check the guard
* If we have a guard, use it to determine if we should run again

We determine if we have run it before though, at the end of the build step, writing
out a cache with the checksum of the build cookbooks lib, recipes/default.rb, resource,
provider, file and template directories. If any of them has changed, we assume we need
to try again.

The cache has a deadline of 24 hours - when it passes, we run it again no matter what.

The guard statement is optional.

## Node Attributes

When you run `delivery job` a `dna.json` file specific to your change is created
in your workspace that contains node attributes that you can reference in your
build cookbooks.

### Workspace Details
Attributes in the `node['delivery']['workspace']` namespace provide paths to the
various directories in your change's workspace on your build node.

* `node['delivery']['workspace']['root']`
* `node['delivery']['workspace']['repo']`
* `node['delivery']['workspace']['chef']`
* `node['delivery']['workspace']['cache']`

### Change Details
Attributes in the `node['delivery']['change']` namespace provide details about
this particular job execution.

* `node['delivery']['change']['enterprise']`
* `node['delivery']['change']['organization']`
* `node['delivery']['change']['project']`
* `node['delivery']['change']['pipeline']`
* `node['delivery']['change']['change_id']`
* `node['delivery']['change']['patchset_number']`
* `node['delivery']['change']['stage']`
* `node['delivery']['change']['phase']`
* `node['delivery']['change']['git_url']`
* `node['delivery']['change']['sha']`
* `node['delivery']['change']['patchset_branch']`

### Project Configuration Details
The contents of your `.delivery/config.json` file are made available to you in the
`node['delivery']['config']` namespace.

## Development

While the Rust Language is now moving towards 1.0, and things should begin to stabilize, follow-on releases sometimes introduce non-backwardly-compatable changes, which can break this build. Until Rust truly stabilizes, you'll need to install rust (the easiest way on mac):

```bash
$ sudo ./rustup.sh --date=2015-04-01 --channel=nightly
```

If this repo fails to build, using the instructions below, you might try:

```bash
$ cargo clean
$ cargo update
```

This may update the Cargo.lock file, which is currently checked in. If there are changes, they should likely be included in your CR.

If there are syntax or other errors, well, good luck!

Note that you can set the logging level by exporting the `RUST_LOG` env var
(e.g to `debug`).

## Build me

```bash
cargo build
```

## Test me

```bash
cargo test
```

## Develop me

Hack about, then:

```bash
cargo run -- review ...
```

Where "review" and friends are the arguments you would pass to the delivery cli.

## License & Authors

- Author:: Adam Jacob (<adam@chef.io>)
- Author:: Seth Falcon (<seth@chef.io>)
- Author:: Jean Rouge (<jean@chef.io>)
- Author:: Tom Duffield (<tom@chef.io>)
- Author:: Jon Anderson (<janderson@chef.io>)

```text
Copyright:: 2015 Chef Software, Inc

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```
