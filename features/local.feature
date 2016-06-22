Feature: local

Scenario: When local --help is run
  When I run `delivery local --help`
  Then the output should contain "SUBCOMMANDS:"
  Then the output should contain "deploy"
  Then the output should contain "lint"
  Then the output should contain "provision"
  Then the output should contain "syntax"
  Then the output should contain "unit"
  And the exit status should be 0

Scenario: When local is run with no subcommands
  When I run `delivery local`
  Then the output should contain "You did not pass a subcommand to"
  And the exit status should be 1

Scenario: When local is run with an invalid subcommand
  When I run `delivery local bogus`
  Then the output should contain "You passed subcommand 'bogus' to"
  And the exit status should be 1
