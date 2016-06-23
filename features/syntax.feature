Feature: syntax

Background:
  Given I clean up the ruby env so I can run other ruby bins like ChefDK
  When I successfully run `chef generate cookbook testbook`

Scenario: When syntax is run on a valid cookbook with -f any
  When I append to "testbook/metadata.rb" with:
  """
  issues_url 'bogus'
  source_url 'bogus'
  """
  When I run `delivery local syntax -f any testbook`
  Then "chef exec foodcritic -f any testbook" should be run
  And the exit status should be 0

Scenario: When syntax is run on an invalid cookbook with -f any
  Given a file named "recipes/default.rb" with:
    """
    str = "str"
    str2 = #{str}
    """
  When I run `delivery local syntax -f any testbook`
  Then "chef exec foodcritic -f any testbook" should be run
  Then the output should contain "FC064: Ensure issues_url is set in metadata"
  And the exit status should be 3

Scenario: When syntax is run on a valid cookbook with no input
  When I append to "testbook/metadata.rb" with:
  """
  issues_url 'bogus'
  source_url 'bogus'
  """
  When I run `delivery local syntax`
  Then "chef exec foodcritic . --exclude spec -f any" should be run
  And the exit status should be 0

Scenario: When syntax is run on an invalid cookbook with no input
  Given a file named "recipes/default.rb" with:
    """
    str = "str"
    str2 = #{str}
    """
  When I run `delivery local syntax`
  Then "chef exec foodcritic . --exclude spec -f any" should be run
  Then the output should contain "FC064: Ensure issues_url is set in metadata"
  And the exit status should be 3

Scenario: When syntax --help is run
  When I run `delivery local syntax --help`
  Then the output should contain "foodcritic [cookbook_paths]"
  And the exit status should be 0
