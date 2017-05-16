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
  And 'git remote add delivery ssh://cukes@skunkworks@delivery.mycompany.com:2828/skunkworks/engineering/phoenix_project' should not be run

Scenario: Specifying the patchset branch explicitly
  When I successfully run `delivery job verify syntax --project phoenix_project --for master --change-id 822b0eee-5cfb-4b35-9331-c9bc6b49bdb2 --branch username/feature/branch`
  Then the exit status should be 0
  And the output should contain:
  """
  Chef Client finished
  """
  And "git clone ssh://cukes@skunkworks@delivery.mycompany.com:2828/skunkworks/engineering/phoenix_project ." should be run
  And 'git fetch origin username/feature/branch' should be run
  And 'git remote add delivery ssh://cukes@skunkworks@delivery.mycompany.com:2828/skunkworks/engineering/phoenix_project' should not be run

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
  And 'git remote add delivery ssh://cukes@skunkworks@delivery.mycompany.com:2828/skunkworks/engineering/phoenix_project' should not be run

Scenario: Executing a local job
  When I run `delivery job verify syntax -l`
  Then the output should contain:
  """
  Chef Client finished
  """
  And 'git remote add delivery ssh://cukes@skunkworks@delivery.mycompany.com:2828/skunkworks/engineering/phoenix_project' should not be run
  And 'git remote add delivery ssh://you@local@localhost:2828/local/workstation/delivery-cli-init' should not be run
  And "git clone ssh://cukes@skunkworks@delivery.mycompany.com:2828/skunkworks/engineering/phoenix_project ." should not be run
  And 'git fetch origin username/feature/branch' should not be run
  And 'git fetch origin master' should be run
  And the exit status should be 0

Scenario: Real job triggering; this command is exactly as we trigger jobs in Chef Automate
  Given I am in a blank workspace
  When I run `delivery job build syntax --server delivery.mycompany.com --user cukes --ent skunkworks --org engineering --project phoenix_project --for master --change-id 80983bb0-5cb5-4ec9-a5f1-b023d4c14d69 --shasum 88782dfd260a2b8277b100ba5192c7131b81aa0a --git-url ssh://cukes@skunkworks@delivery.mycompany.com:2828/skunkworks/engineering/phoenix_project`
  Then the output should contain:
  """
  Chef Client finished
  """
  And "git clone ssh://cukes@skunkworks@delivery.mycompany.com:2828/skunkworks/engineering/phoenix_project ." should be run
  And 'git fetch origin' should be run
  And 'git remote add delivery ssh://cukes@skunkworks@delivery.mycompany.com:2828/skunkworks/engineering/phoenix_project' should not be run
  And the exit status should be 0

Scenario: Real job triggering; If you try to run a job outside of the git_repo and
          also without specifying a `--project` flag. We must fail.
  Given I am in a blank workspace
  When I run `delivery job build syntax --server delivery.mycompany.com --user cukes --ent skunkworks --org engineering --for master --change-id 80983bb0-5cb5-4ec9-a5f1-b023d4c14d69 --shasum 88782dfd260a2b8277b100ba5192c7131b81aa0a --git-url ssh://cukes@skunkworks@delivery.mycompany.com:2828/skunkworks/engineering/phoenix_project`
  Then the output should contain:
  """
  Cannot find a .git/config file. Run 'git init' in your project root to initialize it
  """
  And the exit status should not be 0
