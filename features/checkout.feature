Feature: checkout

  The `checkout` command creates a local branch tracking a remote
  review branch. This allows developers to get a local copy of a
  change that is in-flight. The default is to checkout the latest
  patchset for the change, but a specific patchset can be specified.

Background:
  Given I am in the "project" git repo
  Given I have a valid cli.toml file
  Given I set the environment variables to:
    | variable           | value      |
    | MOCK_ALL_BASH      | true       |
  And a file named ".delivery/config.json" with:
    """
    {
     "version": "1",
     "build_cookbook": "delivery_truck"
     }
    """

Scenario: Happy Path Checkout
  When I successfully run `delivery checkout awesome/feature`
  Then the output should contain "awesome/feature"
  And "git fetch delivery" should be run
  And "git branch --track awesome/feature delivery/_reviews/master/awesome/feature/latest" should be run
  And "git remote add delivery ssh://user@ent@server.test:8989/ent/org/project" should not be run
  And "git checkout awesome/feature" should be run

Scenario: Checkout a change when FIPS mode is enabled
  When I run `delivery checkout awesome/feature --fips --fips-git-port 1234`
  Then the output should contain "Updating 'delivery' remote with the default configuration loaded from"
  And the delivery remote should exist
  And the output should contain "update:  ssh://user@ent@localhost:1234/ent/org/project"
