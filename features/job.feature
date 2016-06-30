Feature: job

Background:
  # `git merge` and `git reset` need to be mocked here because the specific
  # merge and reset commands rely on the existence of an upstream remote
  Given I set the environment variables to:
    | variable             | value      |
    | MERGE_MOCKED         | true       |
    | RESET_MOCKED         | true       |
  And I clean up the ruby env so I can run other ruby bins like ChefDK
  And I am in the "delivery-cli-init" git repo
  And a file named ".delivery/cli.toml" with:
  """
    git_port = "2828"
    pipeline = "master"
    user = "cukes"
    server = "delivery.mycompany.com"
    enterprise = "skunkworks"
    organization = "engineering"
  """

Scenario: With all information specified in the configuration file
  When I successfully run `delivery job verify syntax --project phoenix_project --for master --patchset 1 --change-id 822b0eee-5cfb-4b35-9331-c9bc6b49bdb2 --change username/feature/branch`
  Then the exit status should be 0
  And the output should contain:
  """
  Chef Client finished
  """
  And "git clone ssh://cukes@skunkworks@delivery.mycompany.com:2828/skunkworks/engineering/phoenix_project ." should be run
  And 'git fetch origin _reviews/master/username/feature/branch/1' should be run

Scenario: Specifying the patchset branch explicitly
  When I successfully run `delivery job verify syntax --project phoenix_project --for master --change-id 822b0eee-5cfb-4b35-9331-c9bc6b49bdb2 --branch username/feature/branch`
  Then the exit status should be 0
  And the output should contain:
  """
  Chef Client finished
  """
  And "git clone ssh://cukes@skunkworks@delivery.mycompany.com:2828/skunkworks/engineering/phoenix_project ." should be run
  And 'git fetch origin username/feature/branch' should be run

Scenario: A repo that has failing tests
  Given I have a repository with failing tests
  When I run `delivery job verify syntax --project phoenix_project --for master --change-id 822b0eee-5cfb-4b35-9331-c9bc6b49bdb2 --branch username/feature/branch`
  Then the exit status should not be 0
  And the output should contain:
  """
  Chef Client failed
  """
  And "git clone ssh://cukes@skunkworks@delivery.mycompany.com:2828/skunkworks/engineering/phoenix_project ." should be run
  And 'git fetch origin username/feature/branch' should be run
