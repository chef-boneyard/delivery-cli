Feature: cleanup

Background:
  Given I clean up the ruby env so I can run other ruby bins like ChefDK
  When I successfully run `chef generate cookbook testbook`
  When I cd to "testbook"

Scenario: When cleanup is run on a valid cookbook
  Given I set the environment variables to:
    | variable               | value |
    | MOCK_CHEF_EXEC_KITCHEN | true  |
  When I run `delivery local cleanup`
  Then "chef exec kitchen destroy" should be run
  And the exit status should be 0

Scenario: When cleanup is run on an invalid cookbook
  Given I set the environment variables to:
    | variable                           | value |
    | MOCK_CHEF_EXEC_KITCHEN             | true  |
    | MOCK_CHEF_EXEC_KITCHEN_SHOULD_EXIT | 100   |
  When I run `delivery local cleanup --bogus`
  Then "chef exec kitchen destroy --bogus" should be run
  And the exit status should be 100

Scenario: When cleanup --help is run
  When I run `delivery local cleanup --help`
  Then the output should contain "delivery local cleanup [INSTANCE|REGEXP|all]"
  And the exit status should be 0
