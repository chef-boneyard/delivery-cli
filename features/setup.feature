Feature: setup

  The `setup` command will create a `.delivery/cli.toml` file in the
  current directory, which stores configuration values such as a
  project's enterprise, organization, pipeline, as well as the user's
  name. Other commands can use this, allowing command line invocations
  to be shorter, and data to be reused.

Background:
  Given a directory named ".delivery" should not exist
  And a file named ".delivery/cli.toml" should not exist

Scenario: setup with no additional data supplied

  Without additional arguments, a minimal configuration file is
  written, containing only the `git_port` (which is currently not
  customizable), and the `pipeline` (which defaults to "master").

  When I successfully run `delivery setup`
  Then a directory named ".delivery" should exist
  And a file named ".delivery/cli.toml" should exist
  And the file ".delivery/cli.toml" should contain exactly:
    """
    api_protocol = "https"
    git_port = "8989"
    pipeline = "master"
    a2_mode = false

    """

Scenario: setup with enterprise
  When I successfully run `delivery setup --ent=Foobar`
  Then a directory named ".delivery" should exist
  And a file named ".delivery/cli.toml" should exist
  And the file ".delivery/cli.toml" should contain exactly:
    """
    api_protocol = "https"
    enterprise = "Foobar"
    git_port = "8989"
    pipeline = "master"
    a2_mode = false

    """

Scenario: setup with organization
  When I successfully run `delivery setup --org=Engineering`
  Then a directory named ".delivery" should exist
  And a file named ".delivery/cli.toml" should exist
  And the file ".delivery/cli.toml" should contain exactly:
    """
    api_protocol = "https"
    organization = "Engineering"
    git_port = "8989"
    pipeline = "master"
    a2_mode = false

    """

Scenario: setup with user
  When I successfully run `delivery setup --user=alice`
  Then a directory named ".delivery" should exist
  And a file named ".delivery/cli.toml" should exist
  And the file ".delivery/cli.toml" should contain exactly:
    """
    api_protocol = "https"
    user = "alice"
    git_port = "8989"
    pipeline = "master"
    a2_mode = false

    """

Scenario: setup with pipeline
  When I successfully run `delivery setup --for=legacy`
  Then a directory named ".delivery" should exist
  And a file named ".delivery/cli.toml" should exist
  And the file ".delivery/cli.toml" should contain exactly:
    """
    api_protocol = "https"
    git_port = "8989"
    pipeline = "legacy"
    a2_mode = false

    """

Scenario: setup with server
  When I successfully run `delivery setup --server=delivery.mycompany.com`
  Then a directory named ".delivery" should exist
  And a file named ".delivery/cli.toml" should exist
  And the file ".delivery/cli.toml" should contain exactly:
    """
    server = "delivery.mycompany.com"
    api_protocol = "https"
    git_port = "8989"
    pipeline = "master"
    a2_mode = false

    """

Scenario: setup with project
  When I successfully run `delivery setup --project=coffee_lover`
  Then a directory named ".delivery" should exist
  And a file named ".delivery/cli.toml" should exist
  And the file ".delivery/cli.toml" should contain exactly:
    """
    api_protocol = "https"
    project = "coffee_lover"
    git_port = "8989"
    pipeline = "master"
    a2_mode = false

    """

# A2 projects can only be authenticated with SAML enabled
# therefor by enabling a2-mode, it automatically enabled saml
Scenario: setup an A2 project
  When I successfully run `delivery setup --project=coffee_lover --a2-mode`
  Then a directory named ".delivery" should exist
  And a file named ".delivery/cli.toml" should exist
  And the file ".delivery/cli.toml" should contain exactly:
    """
    api_protocol = "https"
    project = "coffee_lover"
    git_port = "8989"
    pipeline = "master"
    saml = true
    a2_mode = true

    """

Scenario: setup with all the args
  When I successfully run `delivery setup --ent=Foobar --org=Engineering --for=legacy --server=delivery.mycompany.com --user=alice --project=makeitwork`
  Then a directory named ".delivery" should exist
  And a file named ".delivery/cli.toml" should exist
  And the file ".delivery/cli.toml" should contain exactly:
    """
    server = "delivery.mycompany.com"
    api_protocol = "https"
    user = "alice"
    enterprise = "Foobar"
    organization = "Engineering"
    project = "makeitwork"
    git_port = "8989"
    pipeline = "legacy"
    a2_mode = false

    """

Scenario: setup when a config file already exists

    If a configuration file is already present, invoking `setup` will
    simply overlay the given arguments on top of what is already
    present in the file, writing the new configuration out.

  Given a directory named ".delivery"
  And a file named ".delivery/cli.toml" with:
    """
    server = "delivery.mycompany.com"
    api_protocol = "https"
    user = "alice"
    enterprise = "Foobar"
    organization = "Engineering"
    git_port = "8989"
    pipeline = "master"

    """
  When I successfully run `delivery setup --ent=Bar`
  Then the file ".delivery/cli.toml" should contain exactly:
    """
    server = "delivery.mycompany.com"
    api_protocol = "https"
    user = "alice"
    enterprise = "Bar"
    organization = "Engineering"
    git_port = "8989"
    pipeline = "master"
    a2_mode = false

    """
