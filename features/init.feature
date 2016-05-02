Feature: init

Background:
  Given a file named ".delivery/cli.toml" with:
  """
    git_port = "2828"
    pipeline = "master"
    user = "cukes"
    server = "delivery.mycompany.com"
    enterprise = "skunkworks"
    organization = "engineering"
  """

Scenario: When creating a delivery backed project
  When a user creates a delivery backed project
  Then a bitbucket project is created in delivery
  And a change configuring delivery is created
  And the change has the default generated build_cookbook

Scenario: When creating a bitbucket backed project
  When a user creates a bitbucket backed project
  Then a bitbucket project is created in delivery
  And a change configuring delivery is created
  And the change has the default generated build_cookbook

Scenario: When creating a github backed project
  When a user creates a github backed project
  Then a github project is created in delivery
  And a change configuring delivery is created
  And the change has the default generated build_cookbook
