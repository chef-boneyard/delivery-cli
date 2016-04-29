Feature: init

Background:
  Given a file named ".delivery/api-tokens" with:
    """
    127.0.0.1:8080,dummy,dummy|this_is_a_fake_token
    """
  And a dummy Delivery API server
  And I am in the "delivery-cli-init" git repo
  And a file named ".delivery/cli.toml" with:
    """
      git_port = "8080"
      pipeline = "master"
      user = "dummy"
      server = "127.0.0.1:8080"
      enterprise = "dummy"
      organization = "dummy"
    """
  And a file named ".git/config" with:
    """
    [config]
    """

Scenario: When creating a delivery backed project
  When a user creates a delivery backed project
  Then a delivery project is created in delivery
  And a change configuring delivery is created
  And the change has the default generated build_cookbook
  And the exit status should be 0

Scenario: When creating a bitbucket backed project
  When a user creates a bitbucket backed project
  Then a bitbucket project is created in delivery
  And a change configuring delivery is created
  And the change has the default generated build_cookbook
  And the exit status should be 0

Scenario: When creating a github backed project
  When a user creates a github backed project
  Then a github project is created in delivery
  And a change configuring delivery is created
  And the change has the default generated build_cookbook
  And the exit status should be 0

Scenario: When trying to specify both github and bitbucket
  When I run `delivery init --github proj --bitbucket proj`
  Then the output should contain "specify just one Source Code Provider: delivery(default), github or bitbucket"
  And the exit status should be 1

# Pending
Scenario: When skipping the build-cookbook generator
  When a user creates a delivery backed project
  And specifies the option "--skip-build-cookbook"
  Then a delivery project is created in delivery
  And no build-cookbook is generated

Scenario: When providing a custom config.json
  When a user creates a project with a custom config.json
  Then the output should contain "Copying configuration to"
  And a change configuring a custom delivery is created
  And the exit status should be 0
