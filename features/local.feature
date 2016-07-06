Feature: local

Scenario: When local --help is run
  When I run `delivery local --help`
  Then the output should contain "cleanup"
  Then the output should contain "deploy"
  Then the output should contain "lint"
  Then the output should contain "provision"
  Then the output should contain "smoke"
  Then the output should contain "syntax"
  Then the output should contain "unit"
  And the exit status should be 0

Scenario: When local is run with no subcommands
  When I run `delivery local`
  Then the output should contain "error: The following required arguments were not provided:"
  And the exit status should be 1

Scenario: When local is run with an invalid subcommand
  When I run `delivery local bogus`
  Then the output should contain "error: 'bogus' isn't a valid value for '<phase>'"
  And the exit status should be 1
