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
  When I successfully run `git remote add delivery fake`
  Then I run `delivery init`
  Then the output should contain "A git remote named 'delivery' already exists in this repo, but it is different than what was contained in your config file"
  And the exit status should be 1

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
  And the output should contain "git remote add"
  And the exit status should be 0
  And I should be checked out to a feature branch named "initialize-delivery-pipeline"

Scenario: When creating a github backed project with an initial origin remote set
  When I successfully run `git init`
  When I successfully run `git remote add origin fake`
  When a user creates a github backed project
  Then a github project is created in delivery
  And a default config.json is created
  And the change has the default generated build_cookbook
  And the output should not contain "git remote add"
  And the exit status should be 0
  And I should be checked out to a feature branch named "initialize-delivery-pipeline"

Scenario: When trying to specify both github and bitbucket
  When I run `delivery init --github proj --bitbucket proj`
  Then the output should contain "specify just one Source Code Provider: delivery (default), github or bitbucket"
  And the exit status should be 1

Scenario: When skipping the build_cookbook generator
  When I already have a .delivery/config.json on disk
  When a user creates a delivery backed project with option "--skip-build-cookbook"
  Then a delivery project is created in delivery
  And no build_cookbook is generated
  And the exit status should be 0
  And I should be checked out to a feature branch named "initialize-delivery-pipeline"
  And a change should be created for branch "initialize-delivery-pipeline"

Scenario: When specifying a local build_cookbook generator
  When I already have a .delivery/config.json on disk
  Given I have a custom generator cookbook
  When a user creates a delivery backed project with option "--generator /tmp/test-generator"
  Then a delivery project is created in delivery
  And a custom build_cookbook is generated from "local_path"
  And the exit status should be 0

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
  When I have a custom generator cookbook
  When I run `delivery init --generator /tmp/test-generator`
  Then the output should contain "You used a custom build cookbook generator, but .delivery/config.json was not created."
  And the exit status should be 1

Scenario: When providing a custom config.json
  When a user creates a project with a custom config.json
  Then the output should match /Custom Delivery config copied from .* to .*/
  Then the output should contain "Custom delivery config committed to feature branch."
  And the change has the default generated build_cookbook
  And a change configuring a custom delivery is created
  And the exit status should be 0
  And I should be checked out to a feature branch named "add-delivery-config"
  And a change should be created for branch "add-delivery-config"

Scenario: When creating a delivery backed project for a pipeline
          different than master that doesn't exist locally
  When I set up basic delivery and git configs
  Then I run `delivery init --for sad_panda`
  And the output should contain "A sad_panda branch does not exist locally"
  And the exit status should be 1

# This test is currently broken
#
# When we run `delivery init` for a different pipeline that is not master
# we would expect the repository to be already inside the pipeline-branch
# and also to have at least one commit. (as usual)
#
# Problem: ChefDK internally is taking care of the repository manipulation,
# that is, adding the build_cookbook, delivery config.json and more. When it
# is done with the modification it also commit the changes to master and
# push the branch to the Delivery Server. As this code is currently hard
# codded to be always pointing to `master`
# => https://github.com/chef/chef-dk/blob/master/lib/chef-dk/skeletons/code_generator/recipes/build_cookbook.rb#L146
# 
# We have two options to fix this:
# 1) Pass an option to chefdk generate with the branch-pipeline name 
# 2) Remove the git commands from chefdk and let the delivery-cli handle them
#
# TODO: Uncomment this like once we fix this problem
@broken
Scenario: When creating a delivery backed project for a pipeline
	  different than master, we would expect to have at least
	  one commit into the pipeline branch locally
  When I set up basic delivery and git configs
  And I successfully run `git checkout -b awesome`
  Then I run `delivery init --for awesome`
  And the output should match /Pushing local content to.*awesome.*pipeline on server/
  And the exit status should be 0
