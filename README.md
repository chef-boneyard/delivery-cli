# Delivery CLI

A command line tool for continuous delivery workflow. The `delivery`
command is a component of Chef Delivery. It can be used to setup and
execute phase jobs as well as interact with a Chef Delivery server.

It is a part of the ChefDK and can be downloaded [here](https://downloads.chef.io/chef-dk/).

## Getting Started With Delivery

+ [Delivery CLI Docs](https://docs.chef.io/ctl_delivery.html)
+ [Delivery Docs](https://docs.chef.io/start_delivery.html)

In particular, you will want to be familar with:

+ [Your First Delivery Project](https://docs.chef.io/delivery_truck.html#validate-the-installation)
+ [The Delivery Setup Command](https://docs.chef.io/ctl_delivery.html#delivery-setup)
+ [Delivery config.json](https://docs.chef.io/config_json_delivery.html)

## Development

To get started make sure you have the following installed:
+ [Homebrew](http://brew.sh/)
+ [Ruby 2.1.5](https://github.com/rbenv/rbenv)
+ Rust 1.9.0 (`brew install rust`)
+ Openssl (`brew install openssl`)
+ [ChefDK 15.15 or later](https://downloads.chef.io/chef-dk/)

Main technologies used in this project:
+ Rust (learn more [here](http://doc.rust-lang.org/book/installing-rust.html))
+ [Cucumber](https://cucumber.io/docs) and [Aruba](https://github.com/cucumber/aruba) for functional testing
+ [Clap](https://github.com/kbknapp/clap-rs), a CLI parsing library for Rust

We use [make](https://en.wikipedia.org/wiki/Make_(software)) to perform various development
operations like building and testing. The commands reside in the Makefile, but the Makefile
is _only_ used for development. It is not used by omnibus or our delivery cookbooks.

Make targets:
+ `make` builds the project.
+ `make test` runs the unit and functional tests.
+ `make cucumber` will run the cucumber tests.
+ `make clean` will clean the state of the build.
+ `make update_deps` will clean the project and update the `Cargo.lock` file, 
  this should be run periodically to pull in new deps and at the very least when upgrading to a new version of Rust.
+ `make release` builds the project with the `--release` flag.
+ `make check` builds and runs unit tests.

After you `make` the project, you can execute your compiled binary
by running `target/debug/delivery <delivery_args>`. So, you could run something like

```
$ target/debug/delivery review
```

to test the review command with your code in it.

If, for whatever reason, you want to compile and test with cargo's `--release` flag,
run `make release`, and use `target/release/delivery`, but it will take longer to compile.

### Promoting to Github

Currently, when you hit accept in the Delivery RFR UI, it will merge to delivery master,
but not github master. That happens in delivered/functional.

**TO SHIP YOUR CHANGES, YOU MUST DELIVER THEM AS WELL AS APPOVE THEM VIA THE DELIVERY UI**

ChefDK builds pull from master of delivery-cli on github, so delivering your changes in
the delivery project for delivery-cli will "ship" them to the next build of the ChefDK.

### Tips

+ You can set the logging level by exporting the `RUST_LOG`
  environment variable (e.g `RUST_LOG=debug cargo run`).

+ Export `RUST_BACKTRACE=1` for better stack traces on panics.

+ Run only the embedded unit tests (not functional tests in the
  `tests/` directory nor code embedded in doc comments) via `cargo
  test --lib`. You can also pass a module name to only run matching
  tests (e.g. `cargo test --lib job`).

+ To test a specific cucumber module, after a `bundle install`, you can run `bin/cucumber features/<feature>.feature`.

### Cucumber Testing

We heavily rely on git in the project. Some of the cucumber tests mock out parts of git.
Those mocks exist in `features/support/fakebin/git`.

To mock a git call in your cucumber test, set `<NAME_OF_GIT_COMMAND>_MOCKED` to true in
your cucumber test. For example, `features/job.feature` mocks several git commands with:

```
  Given I set the environment variables to:
    | variable             | value      |
    | CHECKOUT_MOCKED      | true       |
    | MERGE_MOCKED         | true       |
    | CLEAN_MOCKED         | true       |
    | RESET_MOCKED         | true       |
```

You can also mock _all_ of git by setting MOCK_ALL_BASH to true:

```
  Given I set the environment variables to:
    | variable           | value      |
    | MOCK_ALL_BASH      | true       |
```

NOTE THAT A FEW COMMANDS ARE MOCKED BY DEFAULT STILL! We want to change this eventually.
They are:

+ clone
+ push
+ fetch
+ ls-remote
+ show

These commands will never actuall execute until their mocks are refactored out of `features/support/fakebin/`.

### Updating Rust Version

When a new version of Rust comes out and it is on homebrew, it's time to update the Rust
version in this repo. There are a few spots to update:

1. Very top of the Makefile
2. omnibus-software's default version
3. The default attributes for the build cookbook for this project

You should also run `make release` to bump the `Cargo.lock` file to get new versions of our
dependencies.

## Delivery Job Implementation Details

The `delivery job` subcommand is used to execute phase recipes for a
project in a workspace. The Delivery server uses `delivery job` to
execute the phases of each stage in the pipeline. The same command can
be used locally to execute a phase job in the same way. You can also
wire `delivery job` into your own pipeline orchestration if you are
not using the Delivery server.

Jobs are run by specifying the stage and the phase. For example, the
following command will run the unit phase of the verify stage:

```bash
delivery job verify unit --local
```

The `delivery job` subcommand will execute the phase recipe by
carrying out the following steps:

0. Currently, the `job` command assumes the current branch is a
   feature branch with commits that are not on the target branch.

1. Clone the project repository to a workspace. The location can be
   customized using `--job-root`.

2. Merge the feature branch into the target branch. This merge will
   not be pushed anywhere and is often referred to as "the
   hypothetical merge"; it is the merge that would result if this
   change were to be approved at this time.

3. Fetch the build cookbook for the project. The build cookbook
   location is specified in the project's `.delivery/config.json`
   file.

4. Use `berks` to fetch dependencies of the build cookbook (as
   specified in the build cookbook's `Berksfile`).

5. Run `chef-client` in local mode with a run list consisting of only
   the `default` recipe of the build cookbook. The `default` recipe is
   intended for preparing a given node to be a "build node" for the
   project. Typical setup handled in the `default` recipe might
   include installing compilers, test frameworks, and other build and
   test dependencies. The `default` recipe is skipped when
   `delivery job` is invoked by a non-root and when the
   `--skip-default` option is specified.

6. Run `chef-client` in local mode with a run list consisting of only
   the specified phase recipe (e.g. `unit`).

Without the `--local` option, the command will look for configuration
required when interacting with a Delivery server from either
`.delivery/cli.toml` or additional command line options.

For local use, you can run multiple phases within a single run like
so:

```
delivery job verify "lint syntax unit"
```

## Delivery Pipeline For This Project

Omnibus build is how the CLI is built on Delivery build nodes. The omnibus build
setup will use the default recipe of the build cookbook (see `cookbooks/delivery_rust/recipes/default.rb`)
and omnibus builds may need sudo permissions for setup.

### Node Attributes

Attributes specific to the project and change are made available for
use in build cookbook recipes.

The global workspace path is accesssible at `node['delivery']['workspace_path']`

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

## License & Authors

- Author:: Adam Jacob (<adam@chef.io>)
- Author:: Seth Falcon (<seth@chef.io>)
- Author:: Jean Rouge (<jean@chef.io>)
- Author:: Tom Duffield (<tom@chef.io>)
- Author:: Jon Anderson (<janderson@chef.io>)
- Author:: Salim Afiune (<afiune@chef.io>)

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
