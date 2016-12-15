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
  And a custom build_cookbook is generated from "local_path"
  And the exit status should be 0

Scenario: When creating a delivery backed project
  When a user creates a delivery backed project
  Then a delivery project is created in delivery
  And a default config.json is created
  And the change has the default generated build_cookbook
  And the exit status should be 0
  And I should be checked out to a feature branch named "initialize-delivery-pipeline"
  And a change should be created for branch "initialize-delivery-pipeline"

Scenario: When running delivery review twice it should not fail
  When a user creates a delivery backed project
  Then a delivery project is created in delivery
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
  Then I run `delivery init`
  Then a delivery project should not be created in delivery
  And a default config.json is created
  And the change has the default generated build_cookbook
  And the exit status should be 0

Scenario: When creating a delivery backed project but
	  the delivery remote is different.
  When I set up basic delivery and git configs
  When I successfully run `git remote add delivery fake`
  When I successfully run `delivery init`
  When I successfully run `git remote -v`
  # The address is 127.0.0.1:8080:8080 because the server is running on localhost:8080
  # and the git port is 8080. Not what you'd ever see irl.
  Then the output should contain "ssh://dummy@dummy@127.0.0.1:8080:8080/dummy/dummy/delivery-cli-init (fetch)"

Scenario: When creating a delivery backed project that already has a .delivery/build_cookbook directory and .delivery/config.json
  When I already have a .delivery/config.json on disk
  When I successfully run `mkdir .delivery/build_cookbook`
  When a user creates a delivery backed project
  Then a delivery project is created in delivery
  And a default config.json is created
  And the change does not have the default generated build_cookbook
  And the output should contain "Skipping: build cookbook already exists at .delivery/build_cookbook."
  And the exit status should be 0
  And I should be checked out to a feature branch named "initialize-delivery-pipeline"
  And a change should be created for branch "initialize-delivery-pipeline"

Scenario: When creating a delivery backed project that has been git initalized but does not have a master branch
  When I successfully run `rm -rf .git`
  When I successfully run `git init`
  When I run `delivery init`
  And the output should contain "A master branch does not exist locally."
  And the exit status should be 1

Scenario: When creating a delivery backed project that already has a .delivery/config.json directory and no custom config is requested
  When I already have a .delivery/config.json on disk
  When a user creates a delivery backed project
  Then a delivery project is created in delivery
  And a change to the delivery config is not comitted
  And the change has the default generated build_cookbook
  And the exit status should be 0
  And I should be checked out to a feature branch named "initialize-delivery-pipeline"
  And a change should be created for branch "initialize-delivery-pipeline"

Scenario: When creating a bitbucket backed project
  When a user creates a bitbucket backed project
  Then a bitbucket project is created in delivery
  And a default config.json is created
  And the change has the default generated build_cookbook
  And the exit status should be 0
  And I should be checked out to a feature branch named "initialize-delivery-pipeline"
  And a change should be created for branch "initialize-delivery-pipeline"

Scenario: When creating a github backed project
  When a user creates a github backed project
  Then a github project is created in delivery
  And a default config.json is created
  And the change has the default generated build_cookbook
  And the exit status should be 0
  And I should be checked out to a feature branch named "initialize-delivery-pipeline"
  And a change should be created for branch "initialize-delivery-pipeline"

Scenario: When creating a github backed project with an initial origin remote set
  When I successfully run `git init`
  When I successfully run `git remote add origin fake`
  When a user creates a github backed project
  Then a github project is created in delivery
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
  And a default config.json is created
  And the change has the default generated build_cookbook
  And I should be checked out to a feature branch named "initialize-delivery-pipeline"
  And a change should be created for branch "initialize-delivery-pipeline"
  And the output should contain "WARN: This project will be named 'delivery-cli-init', but the repository name is 'not-the-right-repo'."
  And the output should contain "Are you sure this is what you want? y/n:"
  And the exit status should be 0

Scenario: When skipping the build_cookbook generator
  When I already have a .delivery/config.json on disk
  When a user creates a delivery backed project with option "--skip-build-cookbook"
  Then a delivery project is created in delivery
  And no build_cookbook is generated
  And the exit status should be 0
  And I should be checked out to a feature branch named "initialize-delivery-pipeline"
  And a change should be created for branch "initialize-delivery-pipeline"

Scenario: When specifying a GitRepo Url for the build_cookbook generator
  When a custom build cookbook is already downloaded in the cache
  When I already have a .delivery/config.json on disk
  When a user creates a delivery backed project with option "--generator https://github.com/afiune/test-generator"
  Then a delivery project is created in delivery
  And a custom build_cookbook is generated from "git_repo"
  And the exit status should be 0
  And I should be checked out to a feature branch named "add-delivery-config"
  And a change should be created for branch "add-delivery-config"

Scenario: When specifying a local build_cookbook generator with no config
          and passing a custom config
  When I have a custom generator cookbook with no config generator
  When a custom config
  Then a user tries to create a delivery backed project with a custom config and custom generator
  And the output should contain "Your new Delivery project is ready"
  And the exit status should be 0

Scenario: When providing a custom config.json
  When a user creates a project with a custom config.json
  Then a custom config is generated
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
  And the output should match /pushing local commits from branch awesome/
  And the exit status should be 0

Scenario: When creating a delivery backed project for a pipeline using --pipeline
	  different than master, we would expect to have at least
	  one commit into the pipeline branch locally
  When I set up basic delivery and git configs
  And I successfully run `git checkout -b awesome`
  Then I run `delivery init --pipeline awesome`
  And the output should match /pushing local commits from branch awesome/
  And the exit status should be 0

Scenario: When creating a delivery backed project for a pipeline using --pipeline
  	  and --for, we would expect to have at least
  	  one commit into the pipeline branch locally
    When I set up basic delivery and git configs
    And I successfully run `git checkout -b awesome`
    Then I run `delivery init --pipeline awesome --for also_awesome`
    And the exit status should be 1
