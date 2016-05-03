Feature: version

  As a user, I want to know what version of the delivery command line
  tool I have.

Scenario: asking for the version with --version

  When I successfully run `delivery --version`
  Then the output should match /delivery \d+\.\d+\.\d+ \([a-f0-9]+\)/

