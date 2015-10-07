Feature: review

  The `review` command allows developers to push feature branches to
  the Delivery server. This is how new change requests get created,
  and how the "Verify" stage gets kicked off for those changes. In
  short, the entire Delivery workflow starts here.

Background:
  Given I am in the "project" git repo
  And a file named ".delivery/config.json" with:
    """
    {
     "version": "1",
     "build_cookbook": "delivery_truck"
     }
    """

Scenario: The Happy Path

  The "happy path" is simply typing `delivery review`. This
  presumes you want your work to merge into the `master` branch
  (i.e., you're targeting the `master` pipeline for your project).

  When I have a feature branch "foo" off of "master"
  And I checkout the "foo" branch
  And I successfully run `delivery review`
  # Really want this output comparison, but I'm running into issues with ANSI color codes at the moment:
  #   Then the output should contain "Review for change foo targeted for pipeline master"
  Then the output should contain "Review for change "
  And "git push --porcelain --progress --verbose delivery foo:_for/master/foo" should be run
#  And "open XXX" should be run

# Scenario: I'm on a different branch than my pipeline target, but there are no additional commits!

#   I think that this should ultimately fail, but I'm on an airplane
#   right now and can't really test it.

#   When I successfully run `git checkout -b foo master`
#   And I run `delivery review`
#   Then the exit status should be 1
#   Then the output should contain "NOPENOPENOPE"
#   And "git push" should not be run


# this one is broken because it requires us to be able to switch the
# behavior of the fakebin git script such that git branch returns
# output indicating we are on master.
@broken
Scenario: I'm on the target branch I'm trying to push for review on

  If I am on the same branch that my review pipeline is
  targeting, I should not be allowed to create a review.

  When I checkout the "master" branch
  And I run `delivery review --for=master`
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
