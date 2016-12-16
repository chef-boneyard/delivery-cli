Feature: review

  The `review` command allows developers to push feature branches to
  the Delivery server. This is how new change requests get created,
  and how the "Verify" stage gets kicked off for those changes. In
  short, the entire Delivery workflow starts here.

Background:
  Given I am in the "project" git repo
  Given I have a valid cli.toml file
  And a file named ".git/config" with:
    """
    [config]
    """
  And a file named ".delivery/config.json" with:
    """
    {
      "version": "2",
      "build_cookbook": {
        "name": "delivery_test",
        "path": "cookbooks/delivery_test"
      },
      "build_nodes": {
        "default"    : ["name:delivery-builder*"]
      },
      "dependencies": [
        "projectA"
      ]
    }
    """

Scenario: The Happy Path

  The "happy path" is simply typing `delivery review`. This
  presumes you want your work to merge into the `master` branch
  (i.e., you're targeting the `master` pipeline for your project).

  When I have a feature branch "foo" off of "master"
  And I checkout the "foo" branch
  And I successfully run `delivery review`
  Then the output should contain "Review for change "
  And the output should not contain "is a cookbook"
  And "git push --porcelain --progress --verbose delivery foo:_for/master/foo" should be run


Scenario: I want to target a different branch than the pipeline default using
          the --for flag.

  When I have a feature branch "foo" off of "master"
  And I checkout the "foo" branch
  And I successfully run `delivery review --for staging`
  Then the output should contain "Review for change "
  And the output should not contain "is a cookbook"
  And "git push --porcelain --progress --verbose delivery foo:_for/staging/foo" should be run
  And "git remote add delivery ssh://user@ent@server.test:8989/ent/org/project" should be run

Scenario: I want to target a different branch than the pipeline default using
          the --pipeline flag.

  When I have a feature branch "foo" off of "master"
  And I checkout the "foo" branch
  And I successfully run `delivery review --pipeline staging`
  Then the output should contain "Review for change "
  And the output should not contain "is a cookbook"
  And "git push --porcelain --progress --verbose delivery foo:_for/staging/foo" should be run

Scenario: I use the --pipeline and the --for flag to set pipeline values

  When I have a feature branch "foo" off of "master"
  And I checkout the "foo" branch
  And I run `delivery review --pipeline staging --for test`
  Then the exit status should be 1

# Scenario: I'm on a different branch than my pipeline target, but there are no additional commits!

#   I think that this should ultimately fail, but I'm on an airplane
#   right now and can't really test it.

#   When I successfully run `git checkout -b foo master`
#   And I run `delivery review`
#   Then the exit status should be 1
#   # TODO: We really need to fix this since the review exits with code 0
#   Then the output should contain "Could not create new patchset - empty change"
#   And "git push" should not be run

Scenario: I run a review on a directory with a different name than
          the project name specified in the cli.toml

  If a user sets up the project name in the cli.toml we should respect
  it and use it to submit the review to that project instead of the
  directory name where we are located.

  Given I have a valid cli.toml file with with "project = 'special'":
  When I have a feature branch "foo" off of "master"
  And I successfully run `delivery review`
  Then the exit status should be 0
  And "git remote add delivery ssh://user@ent@server.test:8989/ent/org/special" should be run

Scenario: I'm on the target branch I'm trying to push for review on

  If I am on the same branch that my review pipeline is
  targeting, I should not be allowed to create a review.

  When I checkout the "master" branch
  And I run `delivery review --for=master`
  Then the exit status should be 1
  And the output should contain "You cannot target code for review from the same branch as the review is targeted for"
  And "git push" should not be run

  Scenario: I'm on the target branch I'm trying to push for review on using --pipeline

    If I am on the same branch that my review pipeline is
    targeting, I should not be allowed to create a review.

    When I checkout the "master" branch
    And I run `delivery review --pipeline=master`
    Then the exit status should be 1
    And the output should contain "You cannot target code for review from the same branch as the review is targeted for"
    And "git push" should not be run

Scenario: I don't want to open a browser

  By default, `delivery review` will run the `open` utility to open
  the review interface for the new change in your default
  browser. This can be overridden using the `--no-open` flag

  When I have a feature branch "foo" off of "master"
  And I checkout the "foo" branch
  And I successfully run `delivery review --no-open`
  Then "open" should not be run

Scenario: I review a cookbook without the auto_bump feature enabled

  By default, we do not enable the auto_bump feature, so if I submit a
  review of a change in a cookbook without enabling this functionality
  the delivery cli will behave normally without detecting or modifying
  the cookobook itself.

  Given a file named "metadata.rb" with:
    """
    version '1.2.3'
    """
  And I have a feature branch "cookbook" off of "master"
  And I checkout the "cookbook" branch
  When I successfully run `delivery review`
  Then the exit status should be 0
  And "git show master:metadata.rb" should not be run
  And the output should not contain "is a cookbook"
  And the output should not contain "Validating version in metadata"
  And the output should not contain "Version already updated"
  And the output should not match /Bumping version to/
  And the file "metadata.rb" should contain exactly:
    """
    version '1.2.3'
    """

Scenario: I review a cookbook that the version hasn't been bumped

  If I submit a review of a change in a cookbook that
  the version hasn't been bumped and I enable the auto_bump feature,
  the delivery cli will detect it and update it for you.

  Given a file named "metadata.rb" with:
    """
    version '1.2.3'
    """
  And I commit all files with message "existing repo has version 1.2.3"
  And I have a feature branch "cookbook" off of "master"
  And I checkout the "cookbook" branch
  When I successfully run `delivery review --auto-bump`
  Then the exit status should be 0
  And "git show master:metadata.rb" should be run
  And the output should contain "is a cookbook"
  And the output should contain "Validating version in metadata"
  And the output should match /Bumping version to:.*1\.2\.4/
  And the output should contain "1.2.4"
  And the file "metadata.rb" should contain exactly:
    """
    version '1.2.4'
    """

Scenario: I review a cookbook that the version has already been bumped

  If I submit a review of a change in a cookbook that
  the version has already been bumped and I enable the auto_bump feature,
  the delivery cli will detect it and will NOT update.

  Given a file named "metadata.rb" with:
    """
    version '1.2.3'
    """
  And I commit all files with message "old version is 1.2.3"
  And a file named "metadata.rb" with:
    """
    version '1.2.4'
    """
  And I have a feature branch "cookbook" off of "master"
  And I checkout the "cookbook" branch
  When I successfully run `delivery review -a`
  Then the exit status should be 0
  And "git show master:metadata.rb" should be run
  And the output should contain "is a cookbook"
  And the output should contain "Validating version in metadata"
  And the output should contain "Version already updated"
  And the output should not match /Bumping version to/
  And the file "metadata.rb" should contain exactly:
    """
    version '1.2.4'
    """

Scenario: I enable the auto_bump feature persistently in the cli.toml

  If I activate the auto_bump feature persistently and I submit a
  review of a change in a cookbook, the delivery cli will detect it
  and update the version if it hasn't been bumped already.

  Given a file named "metadata.rb" with:
    """
    version '1.2.3'
    """
  And I commit all files with message "existing repo has version 1.2.3"
  And I have a valid cli.toml file with with "auto_bump = true":
  And I have a feature branch "cookbook" off of "master"
  And I checkout the "cookbook" branch
  When I successfully run `delivery review`
  Then the exit status should be 0
  And "git show master:metadata.rb" should be run
  And the output should contain "is a cookbook"
  And the output should contain "Validating version in metadata"
  And the output should match /Bumping version to:.*1\.2\.4/
  And the file "metadata.rb" should contain exactly:
    """
    version '1.2.4'
    """

Scenario: Review from child dir

  When I have a feature branch "foo" off of "master"
  And I checkout the "foo" branch
  And I successfully run `mkdir tmp`
  And I cd to "tmp/"
  And I successfully run `delivery review`
  Then the output should contain "Review for change "
  And the output should not contain "is a cookbook"
  And "git push --porcelain --progress --verbose delivery foo:_for/master/foo" should be run

Scenario: Review with a V1 config

  When I have a feature branch "foo" off of "master"
  And I checkout the "foo" branch
  And a file named ".delivery/config.json" with:
    """
    {
      "version": "1",
      "build_cookbook": "./.delivery/build_cookbook",
      "build_nodes": {
        "default": ["name:delivery-builder*"]
      }
    }
    """
  And I successfully run `delivery review`
  Then the output should contain "Review for change "
  And the output should not contain "is a cookbook"
  And "git push --porcelain --progress --verbose delivery foo:_for/master/foo" should be run
