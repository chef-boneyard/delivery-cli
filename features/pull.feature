Feature: pull

  The `pull` command pulls a branch down from the delivery remote and
  merges or rebases current HEAD onto it.

Background:
  Given I am in the "project" git repo
  Given I have a valid cli.toml file
  And a file named ".git/config" with:
    """
    [config]
    """

Scenario: Merge
  # ls-remote is mocked to know about feature-branch
  When I have a feature branch "feature-branch" off of "master"
  And I checkout the "feature-branch" branch
  And I successfully run `delivery pull feature-branch`
  Then the output should contain "Merging local HEAD on remote version of feature-branch"
  Then "git pull delivery feature-branch" should be run

Scenario: Rebase
  # ls-remote is mocked to know about feature-branch
  When I have a feature branch "feature-branch" off of "master"
  And I checkout the "feature-branch" branch
  And I successfully run `delivery pull feature-branch --rebase`
  Then the output should contain "Rebasing local HEAD on remote version of feature-branch"
  Then "git pull delivery feature-branch --rebase" should be run

Scenario: Branch doesn't exist
  When I have a feature branch "not-on-remote" off of "master"
  And I checkout the "not-on-remote" branch
  And I run `delivery pull not-on-remote --rebase`
  Then the output should contain "A pipeline or branch named not-on-remote was not found on the delivery remote"
  Then "git pull delivery not-on-remote" should not be run