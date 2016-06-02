Feature: job

Background:
  Given I set the environment variables to:
    | variable             | value      |
    | CHECKOUT_MOCKED      | true       |
    | MERGE_MOCKED         | true       |
    | CLEAN_MOCKED         | true       |
    | RESET_MOCKED         | true       |
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
  Then "git clone ssh://cukes@skunkworks@delivery.mycompany.com:2828/skunkworks/engineering/phoenix_project ." should be run
  And 'git fetch origin _reviews/master/username/feature/branch/1' should be run
  And 'chef-client -z --force-formatter -j <%= ENV['HOME'] %>/.delivery/delivery.mycompany.com/skunkworks/engineering/phoenix_project/master/verify/syntax/chef/dna.json -c <%= ENV['HOME'] %>/.delivery/delivery.mycompany.com/skunkworks/engineering/phoenix_project/master/verify/syntax/chef/config.rb -r build_cookbook::syntax' should be run

Scenario: Specifying the patchset branch explicitly
  When I successfully run `delivery job verify syntax --project phoenix_project --for master --change-id 822b0eee-5cfb-4b35-9331-c9bc6b49bdb2 --branch username/feature/branch`
  Then "git clone ssh://cukes@skunkworks@delivery.mycompany.com:2828/skunkworks/engineering/phoenix_project ." should be run
  And 'git fetch origin username/feature/branch' should be run
  And 'chef-client -z --force-formatter -j <%= ENV['HOME'] %>/.delivery/delivery.mycompany.com/skunkworks/engineering/phoenix_project/master/verify/syntax/chef/dna.json -c <%= ENV['HOME'] %>/.delivery/delivery.mycompany.com/skunkworks/engineering/phoenix_project/master/verify/syntax/chef/config.rb -r build_cookbook::syntax' should be run
