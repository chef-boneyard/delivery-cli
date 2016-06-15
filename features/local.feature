Feature: local

Scenario: When local --help is run
  When I run `delivery local --help`
  Then the output should contain "SUBCOMMANDS:\n    lint"
  And the exit status should be 0
