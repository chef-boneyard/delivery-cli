# Delivery CLI

A command line tool for continuous delivery workflow. The `delivery`
command is a component of Chef Delivery. It can be used to setup and
execute phase jobs as well as interact with a Chef Delivery server.

## Usage

Start using `delivery` by issuing the setup command:

```shell
$ delivery setup --user USER --server SERVER --ent ENTERPRISE --org ORGANIZATION --config-path /Users/adam
```

This will configure delivery to, by default, contact the delivery
server at SERVER, with a default ENTERPRISE and ORGANIZATION.

### `delivery job`

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

## The project config file

The `delivery` tool expects to find a JSON configuration file in the
top-level directory of a project located at
`.delivery/config.json`. This file specifies the build cookbook for
the project. It can also be used to pass data that can be read by the
build cookbook recipes. There are additional fields that are used by
the Delivery server to control job dispatch.

You can create a starting config file (as well as a build cookbook)
using the `init` subcommand:

```bash
delivery init --local
```

Example `.delivery/config.json` specifying an embedded build cookbook:

```json
{
  "version": "2",
  "build_cookbook": {
    "path": ".delivery/build-cookbook",
    "name": "build-cookbook"
  },
  "skip_phases": [],
  "build_nodes": {}
}
```

### Specifying a project's build cookbook

The `build_cookbook` field of the config file is used to specify the
build cookbook for the project. Build cookbooks can be fetched from
four sources: local directory within the project, a git repository, a
supermarket instance, or from a Delivery server.

#### From a local directory

```json
"build_cookbook": {
      "name": "delivery_rust",
      "path": "cookbooks/delivery_rust"
}
```

#### From a Git source

```json
"build_cookbook": {
      "name"  : "delivery-truck",
      "git"   : "https://github.com/opscode-cookbooks/delivery-truck.git",
      "branch": "master"
}
```

#### From a Supermarket

```json
"build_cookbook": {
      "name": "delivery-truck",
      "supermarket": "true"
}
```

#### From a Chef Server

```json
"build_cookbook": {
      "name": "delivery-truck",
      "server": "true"
}
```

## Node Attributes

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

## Development

To setup your machine for hacking on `delivery-cli`, follow these
steps. Note that we are currently using Rust nightlies and the the
setup instructions will install the correct build for you.

If you are developing on OS X, make sure you have [homebrew][] and
[ChefDK][] installed.

To get going, run:

```bash
make setup all
```

This will install the proper version of Rust on your system (`make
setup`) using the setup recipe of the CLI's delivery build cookbook
(see `cookbooks/delivery_rust/recipes/default.rb`), and then compile
and build the `delivery` executable (`make all`). You'll find the
executable at `./target/release/delivery`

On OS X, if you've installed Rust before and any directories are owned
by root, you'll run into issues with `make setup`, since it installs
without `sudo`. To fix it, you'll want to uninstall your
previously-installed Rust. Per [the instructions][] run:

    sudo /usr/local/lib/rustlib/uninstall.sh

I also had to manually delete some empty documentation directories
before `make setup` worked for me, as these were also owned by root:

    sudo rmdir /usr/local/share/doc/cargo
    sudo rmdir /usr/local/share/doc/rust

Once you've gotten Rust installed and are doing everyday hacking, you
can just run a simple `make`.

You can run the tests like this:

```bash
make test
make cucumber
```

[homebrew]: http://brew.sh/
[ChefDK]: https://downloads.chef.io/chef-dk/
[the instructions]: http://doc.rust-lang.org/book/installing-rust.html

Tips:

* You can set the logging level by exporting the `RUST_LOG`
  environment variable (e.g `RUST_LOG=debug cargo run`).

* Export `RUST_BACKTRACE=1` for better stack traces on panics.

* Run only the embedded unit tests (not functional tests in the
  `tests/` directory nor code embedded in doc comments) via `cargo
  test --lib`. You can also pass a module name to only run matching
  tests (e.g. `cargo test --lib job`).

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
