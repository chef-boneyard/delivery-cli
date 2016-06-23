Feature: unit

Background:
  Given I clean up the ruby env so I can run other ruby bins like ChefDK
  When I successfully run `chef generate cookbook testbook`
  When I cd to "testbook"

Scenario: When unit is run on a valid cookbook
  When I run `delivery local unit`
  Then the output should contain "1 example, 0 failures"
  And the exit status should be 0

Scenario: When unit is run on an invalid cookbook
  Given a file named "spec/unit/recipes/other_spec.rb" with:
    """
    require 'spec_helper'

    describe 'false' do
      it 'fails' do
        expect(true).to eq(false)
      end
    end
    """
  When I run `delivery local unit`
  Then the output should contain "2 examples, 1 failure"
  And the exit status should be 1

Scenario: When unit --help is run
  When I run `delivery local unit --help`
  Then the output should contain "Usage: rspec [options]"
  And the exit status should be 0
