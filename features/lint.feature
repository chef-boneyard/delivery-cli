Feature: lint

Background:
  Given I clean up the ruby env so I can run other ruby bins like ChefDK
  When I successfully run `which chef`
  When I successfully run `chef generate cookbook testbook`
  Then I successfully run `cd testbook`

Scenario: When lint is run on a valid cookbook
  When I run `delivery local lint`
  Then the output should contain "no offenses detected"
  And the exit status should be 0

Scenario: When lint is run on an invalid cookbook
  Given a file named "recipes/default.rb" with:
    """
    hash = {
      "wrong" => "syntax"
    }
    """
  When I run `delivery local lint`
  Then the output should contain "Offenses:"
  And the exit status should be 1

Scenario: When lint --help is run
  When I run `delivery local lint --help`
  Then the output should contain "Usage: rubocop"
  And the exit status should be 0
