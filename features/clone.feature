Feature: clone

Background:
  Given a file named ".delivery/cli.toml" with:
  """
    git_port = "8989"
    pipeline = "master"
    user = "cukes"
    server = "delivery.mycompany.com"
    enterprise = "skunkworks"
    organization = "engineering"
  """

Scenario: With all information specified in the configuration file
  When I successfully run `delivery clone phoenix_project`
  Then "git clone ssh://cukes@skunkworks@delivery.mycompany.com:8989/skunkworks/engineering/phoenix_project phoenix_project" should be run
  And "git remote add delivery ssh://cukes@skunkworks@delivery.mycompany.com:8989/skunkworks/engineering/phoenix_project" should be run

Scenario: With all information specified in the configuration file, but overriding enterprise
  When I successfully run `delivery clone phoenix_project --ent=alternate`
  Then "git clone ssh://cukes@alternate@delivery.mycompany.com:8989/alternate/engineering/phoenix_project phoenix_project" should be run
  And "git remote add delivery ssh://cukes@alternate@delivery.mycompany.com:8989/alternate/engineering/phoenix_project" should be run

Scenario: With all information specified in the configuration file, but overriding org
  When I successfully run `delivery clone phoenix_project --org=testing`
  Then "git clone ssh://cukes@skunkworks@delivery.mycompany.com:8989/skunkworks/testing/phoenix_project phoenix_project" should be run
  And "git remote add delivery ssh://cukes@skunkworks@delivery.mycompany.com:8989/skunkworks/testing/phoenix_project" should be run

Scenario: With all information specified in the configuration file, but overriding user
  When I successfully run `delivery clone phoenix_project --user=cucumber`
  Then "git clone ssh://cucumber@skunkworks@delivery.mycompany.com:8989/skunkworks/engineering/phoenix_project phoenix_project" should be run
  And "git remote add delivery ssh://cucumber@skunkworks@delivery.mycompany.com:8989/skunkworks/engineering/phoenix_project" should be run

Scenario: With all information specified in the configuration file, but overriding server
  When I successfully run `delivery clone phoenix_project --server=delivery-acceptance.mycompany.com`
  Then "git clone ssh://cukes@skunkworks@delivery-acceptance.mycompany.com:8989/skunkworks/engineering/phoenix_project phoenix_project" should be run
  And "git remote add delivery ssh://cukes@skunkworks@delivery-acceptance.mycompany.com:8989/skunkworks/engineering/phoenix_project" should be run

Scenario: With all information specified in the configuration file, but with a2 mode
  When I successfully run `delivery clone phoenix_project --a2-mode`
  Then "git clone ssh://cukes@skunkworks@delivery.mycompany.com:8989/skunkworks/engineering/phoenix_project phoenix_project" should be run
  And "git remote add delivery ssh://cukes@skunkworks@delivery.mycompany.com:8989/skunkworks/engineering/phoenix_project" should be run

Scenario: With no configuration file and no arguments provided
  Given I remove the file ".delivery/cli.toml"
  When I run `delivery clone phoenix_project`
  Then the exit status should be 1
  And the output should contain "A configuration value is missing"
  And "git clone" should not be run
  And "git remote add" should not be run

@follow-up
Scenario: With all information specified in the configuration file, but specifying --git-url

  If the user specifies a `--git-url`, the CLI will perform the
  initial clone from the given URL, but will still set up the `delivery`
  remote according to configured values.

  Is this the intended behavior? Should we see if that's a real
  `delivery` remote?

  When I successfully run `delivery clone phoenix_project --git-url=https://github.com/chef/chef`
  Then "git clone https://github.com/chef/chef phoenix_project" should be run
  And "git remote add delivery ssh://cukes@skunkworks@delivery.mycompany.com:8989/skunkworks/engineering/phoenix_project" should be run

Scenario: With a non-standard Git port

  The CLI configuration file currently has a `git_port` entry; we
  should honor this if it has changed from the standard `8989`.

  There is not currently a command-line override for `git_port`

  Given a file named ".delivery/cli.toml" with:
  """
    git_port = "2112"
    pipeline = "master"
    user = "cukes"
    server = "delivery.mycompany.com"
    enterprise = "skunkworks"
    organization = "engineering"
  """
  When I successfully run `delivery clone phoenix_project`
  Then "git clone ssh://cukes@skunkworks@delivery.mycompany.com:2112/skunkworks/engineering/phoenix_project phoenix_project" should be run
  And "git remote add delivery ssh://cukes@skunkworks@delivery.mycompany.com:2112/skunkworks/engineering/phoenix_project" should be run

Scenario: Trying to clone a project that already exists
  Given a directory named "phoenix_project"
  When I run `delivery clone phoenix_project`
  Then the exit status should be 1
  And the output should contain "Unable to clone project."
  And the output should match /The destination path (.*)phoenix_project' already exists./
  And "git clone" should not be run
  And "git remote add" should not be run
