Feature: init

Background:
  When I clean up the ruby env so I can run other ruby bins like ChefDK
  Given a generator cookbook cache exists
  Given a file named ".delivery/api-tokens" with:
    """
    127.0.0.1:8080,dummy,dummy|this_is_a_fake_token
    """
  And a dummy Delivery API server
  And I am in the "delivery-cli-init" git repo
  And I set up basic delivery and git configs

Scenario: When specifying a local build_cookbook generator with no config
  Given I have a custom generator cookbook with no config generator
  And a user tries to create a delivery backed project with a custom generator
  Then the output should contain "You used a custom build cookbook generator, but .delivery/config.json was not created"
  And the exit status should be 1

Scenario: When specifying a local build_cookbook generator
  Given I have a custom generator cookbook
  When I already have a .delivery/config.json on disk
  And a user tries to create a delivery backed project with a custom generator
  Then a delivery project is created in delivery
  And the delivery remote should exist
  And a custom build_cookbook is generated from "local_path"
  And the exit status should be 0

Scenario: When creating a delivery backed project
  When a user creates a delivery backed project
  Then a delivery project is created in delivery
  And the delivery remote should exist
  And a default config.json is created
  And the change has the default generated build_cookbook
  And the exit status should be 0
  And I should be checked out to a feature branch named "initialize-delivery-pipeline"
  And a change should be created for branch "initialize-delivery-pipeline"

Scenario: When running delivery review twice it should not fail
  When a user creates a delivery backed project
  Then a delivery project is created in delivery
  And the delivery remote should exist
  And a default config.json is created
  And I set up basic delivery and git configs
  And the change has the default generated build_cookbook
  And the exit status should be 0
  And I should be checked out to a feature branch named "initialize-delivery-pipeline"
  And a change should be created for branch "initialize-delivery-pipeline"
  Then I successfully run `delivery review`

Scenario: When creating a delivery backed project and
	  the project already exists on the server
  When I cd to ".."
  When I am in the "already-created" git repo
  When I set up basic delivery and git configs
  Then I successfully run `delivery init`
  Then a delivery project should not be created in delivery
  And the delivery remote should exist
  And a default config.json is created
  And the change has the default generated build_cookbook
  And the exit status should be 0

Scenario: When creating a delivery backed project that already has a .delivery/build_cookbook directory and .delivery/config.json
  When I already have a .delivery/config.json on disk
  When I successfully run `mkdir .delivery/build_cookbook`
  When a user creates a delivery backed project
  Then a delivery project is created in delivery
  And the delivery remote should exist
  And a default config.json is created
  And the change does not have the default generated build_cookbook
  And the output should contain "Skipping: build cookbook already exists at .delivery/build_cookbook."
  And the exit status should be 0
  And I should be checked out to a feature branch named "initialize-delivery-pipeline"
  And a change should be created for branch "initialize-delivery-pipeline"

Scenario: When creating a delivery backed project that has been git initalized but does not have a master branch
  When I successfully run `rm -rf .git`
  And I successfully run `git init`
  And I run `delivery init`
  Then the output should contain "A master branch does not exist locally."
  And the exit status should be 1

Scenario: When creating a delivery backed project that already has a .delivery/config.json directory and no custom config is requested
  When I already have a .delivery/config.json on disk
  And a user creates a delivery backed project
  Then a delivery project is created in delivery
  And the delivery remote should exist
  And a change to the delivery config is not comitted
  And the change has the default generated build_cookbook
  And the exit status should be 0
  And I should be checked out to a feature branch named "initialize-delivery-pipeline"
  And a change should be created for branch "initialize-delivery-pipeline"

Scenario: When creating a bitbucket backed project
  When a user creates a bitbucket backed project
  Then a bitbucket project is created in delivery
  And the delivery remote should exist
  And a default config.json is created
  And the change has the default generated build_cookbook
  And the exit status should be 0
  And I should be checked out to a feature branch named "initialize-delivery-pipeline"
  And a change should be created for branch "initialize-delivery-pipeline"

Scenario: When creating a github backed project
  When a user creates a github backed project
  Then a github project is created in delivery
  And the delivery remote should exist
  And a default config.json is created
  And the change has the default generated build_cookbook
  And the exit status should be 0
  And I should be checked out to a feature branch named "initialize-delivery-pipeline"
  And a change should be created for branch "initialize-delivery-pipeline"

Scenario: When creating a github backed project with an initial origin remote set
  When I successfully run `git init`
  And I successfully run `git remote add origin fake`
  And a user creates a github backed project
  Then a github project is created in delivery
  And the delivery remote should exist
  And a default config.json is created
  And the change has the default generated build_cookbook
  And the exit status should be 0
  And I should be checked out to a feature branch named "initialize-delivery-pipeline"
  And a change should be created for branch "initialize-delivery-pipeline"

Scenario: When trying to specify both github and bitbucket
  When I run `delivery init --github proj --bitbucket proj`
  Then the output should contain "specify just one Source Code Provider: delivery (default), github or bitbucket"
  And the exit status should be 1

Scenario: When the directory name does not match the repo-name and I abort
  When I run `delivery init --bitbucket chef --repo-name not-the-right-repo` interactively
  And I type "n"
  Then the output should contain "WARN: This project will be named 'delivery-cli-init', but the repository name is 'not-the-right-repo'."
  And the output should contain "Are you sure this is what you want? y/n:"
  And the output should contain "To match the project and the repository name you can:"
  And the output should contain "1) Create a directory with the same name as the repository."
  And the output should contain "2) Clone or download the content of the repository inside."
  And the output should contain "3) Run the 'delivery init' command within the new directory."
  And the exit status should be 1

Scenario: When the directory name does not match the repo-name but I still want to proceed
          we should at least display a WARNING message
  When I run `delivery init --github chef --repo-name not-the-right-repo` interactively
  And I type "y"
  Then a github project is created in delivery
  And the delivery remote should exist
  And a default config.json is created
  And the change has the default generated build_cookbook
  And I should be checked out to a feature branch named "initialize-delivery-pipeline"
  And a change should be created for branch "initialize-delivery-pipeline"
  And the output should contain "WARN: This project will be named 'delivery-cli-init', but the repository name is 'not-the-right-repo'."
  And the output should contain "Are you sure this is what you want? y/n:"
  And the exit status should be 0

Scenario: When skipping the build_cookbook generator
  When I already have a .delivery/config.json on disk
  And a user creates a delivery backed project with option "--skip-build-cookbook"
  Then a delivery project is created in delivery
  And the delivery remote should exist
  And no build_cookbook is generated
  And the exit status should be 0
  And I should be checked out to a feature branch named "initialize-delivery-pipeline"
  And a change should be created for branch "initialize-delivery-pipeline"

Scenario: When specifying a GitRepo Url for the build_cookbook generator
  When a custom build cookbook is already downloaded in the cache
  And I already have a .delivery/config.json on disk
  And a user creates a delivery backed project with option "--generator https://github.com/afiune/test-generator"
  Then a delivery project is created in delivery
  And the delivery remote should exist
  And a custom build_cookbook is generated from "git_repo"
  And the exit status should be 0
  And I should be checked out to a feature branch named "add-delivery-config"
  And a change should be created for branch "add-delivery-config"

Scenario: When specifying a local build_cookbook generator with no config
          and passing a custom config
  When I have a custom generator cookbook with no config generator
  When a custom config
  Then a user tries to create a delivery backed project with a custom config and custom generator
  And the delivery remote should exist
  And the output should contain "Your new Delivery project is ready"
  And the exit status should be 0

Scenario: When providing a custom config.json
  When a user creates a project with a custom config.json
  Then a custom config is generated
  And the delivery remote should exist
  And the change has the default generated build_cookbook
  And a change configuring a custom delivery is created
  And the exit status should be 0
  And I should be checked out to a feature branch named "add-delivery-config"
  And a change should be created for branch "add-delivery-config"

Scenario: When specifying both, generator and custom config we will expect
          the custom generator to write both, the cookbook and the config,
          and then the custom config provided would be overwritten
  When a user creates a project with both a custom generator and custom config
  Then a delivery project is created in delivery
  And the delivery remote should exist
  And both a custom build_cookbook and custom config is generated
  And the change has the default generated build_cookbook
  And the exit status should be 0
  And I should be checked out to a feature branch named "add-delivery-config"
  And a change should be created for branch "add-delivery-config"

Scenario: When creating a delivery backed project for a pipeline
          different than master that doesn't exist locally
  When I set up basic delivery and git configs
  Then I run `delivery init --for sad_panda`
  And the output should contain "A sad_panda branch does not exist locally"
  And the exit status should be 1

Scenario: When creating a delivery backed project for a pipeline
	  different than master, we would expect to have at least
	  one commit into the pipeline branch locally
  When I set up basic delivery and git configs
  And I successfully run `git checkout -b awesome`
  Then I run `delivery init --for awesome`
  And the delivery remote should exist
  And the output should match /pushing local commits from branch awesome/
  And the exit status should be 0

Scenario: When creating a delivery backed project for a pipeline using --pipeline
	  different than master, we would expect to have at least
	  one commit into the pipeline branch locally
  When I set up basic delivery and git configs
  And I successfully run `git checkout -b awesome`
  Then I run `delivery init --pipeline awesome`
  And the delivery remote should exist
  And the output should match /pushing local commits from branch awesome/
  And the exit status should be 0

Scenario: When creating a delivery backed project for a pipeline using --pipeline
  	  and --for, we would expect to have at least
  	  one commit into the pipeline branch locally
  When I set up basic delivery and git configs
  And I successfully run `git checkout -b awesome`
  Then I run `delivery init --pipeline awesome --for also_awesome`
  And the exit status should be 1

Scenario: When initializing a project that already have a custom config
          that source the build cookbook from Supermarket, it should not
	  create any build cookbook locally.
  When I have a config where the build_cookbook comes from Supermarket
  When a user creates a delivery backed project
  Then a delivery project is created in delivery
  And the delivery remote should exist
  And the change does not have the default generated build_cookbook
  And the output should contain "Skipping: build cookbook doesn't need to be generated locally."
  And the exit status should be 0
  And I should be checked out to a feature branch named "initialize-delivery-pipeline"
  And a change should be created for branch "initialize-delivery-pipeline"

# For now, the chef-dk generate build-cookbook command won't let you pass a custom
# path where it should render the build_cookbook, until that is possible we will be
# testing to fail when the path is not the default.
# TODO: (IDEA#383) Be able to generate build-cookbooks on a custom location
#
# Happy path: The custom config has a default .delivery/build_cookbook
Scenario: When initializing a project that already have a custom config
          that defines a local build cookbook location, it should create
          a build cookbook locally if it is the default location.
  When I have already a custom config
  When a user creates a delivery backed project
  Then a delivery project is created in delivery
  And the delivery remote should exist
  And the change has a generated build_cookbook called ".delivery/build_cookbook"
  And the output should contain "Build cookbook generated at .delivery/build_cookbook."
  And the exit status should be 0
  And I should be checked out to a feature branch named "initialize-delivery-pipeline"
  And a change should be created for branch "initialize-delivery-pipeline"

# Angry path: The custom config has a different location for the build_cookbook path
Scenario: When initializing a project that already have a custom config
          that defines a local build cookbook location that is NOT the
          default, error with a nice message
  When I have already a custom config with a custom build_cookbook path
  When I run `delivery init`
  Then the exit status should be 1
  And the output should contain "No valid build_cookbook entry in .delivery/config.json"
  And the output should contain "The build_cookbook cookbooks/bubulubu doesn't exist."
