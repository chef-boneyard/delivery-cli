Feature: local

  The `delivery local` command runs Workflow phases and stages on your
  local workstation, it requires your project to have the `project.toml`
  file inside the `.delivery/` directory where a user can configure the
  command(s) to run.

Background:
  When I clean up the ruby env so I can run other ruby bins like ChefDK
  Given I am in a chefdk generated cookbook called "local"

Scenario: When local --help is run
  When I run `delivery local --help`
  Then the output should contain "cleanup"
  And the output should contain "deploy"
  And the output should contain "lint"
  And the output should contain "provision"
  And the output should contain "smoke"
  And the output should contain "functional"
  And the output should contain "syntax"
  And the output should contain "unit"
  And the exit status should be 0

Scenario: When local is run with no subcommands
  When I run `delivery local`
  Then the output should contain "error: The following required arguments were not provided:"
  And the exit status should be 1

Scenario: When local is run with an invalid subcommand
  When I run `delivery local bogus`
  Then the output should contain "error: 'bogus' isn't a valid value for '<stage_phase>'"
  And the exit status should be 1

Scenario: Executing the lint phase locally
  When I run `delivery local lint`
  Then the output should match /Running.*Lint.*Phase/
  And the output should contain "no offenses detected"
  And the exit status should be 0

Scenario: Executing the syntax phase locally
  When I run `delivery local syntax`
  Then the output should match /Running.*Syntax.*Phase/
  And the exit status should be 0

# Uncomment this test once we can add chef-dk as a gem dependency
# inside the Gemfile, if you run this locally it actually works:
#
# Behind the scenes you could run:
# ```
# chef generate cookbook local
# cd local
# delivery local unit
# ```
#
#Scenario: Executing the unit phase locally
  #When I invoke a pseudo tty with command "delivery local unit"
  #And I want to debug the pseudo tty command
  #And I cd inside my ptty to "local"
  #And I run my ptty command
  #Then the ptty output should contain "Running.*Unit.*Phase"
  #And the ptty output should contain "0 failures"
  #And the ptty exit status should be 0

Scenario: Verify that when we modify the `.delivery/project.toml`
          the `delivery local` command picks it up
  When I have a custom project.toml file
  And I invoke a pseudo tty with command "delivery local unit"
  And I cd inside my ptty to "local"
  And I run my ptty command
  Then the ptty output should contain "This is a cool unit test"
  And the ptty exit status should be 0

Scenario: When the project has an invalid `.delivery/project.toml`
  When I have an incomplete project.toml file
  And I invoke a pseudo tty with command "delivery local lint"
  And I want to debug the pseudo tty command
  And I cd inside my ptty to "local"
  And I run my ptty command
  Then the ptty exit status should be 1
  And the ptty output should contain "LocalPhases tag not found"

Scenario: When the project has a partial `.delivery/project.toml`
	  it should not fail to execute the configured phase
  When I have a partially config project.toml file
  And I invoke a pseudo tty with command "delivery local lint"
  And I want to debug the pseudo tty command
  And I cd inside my ptty to "local"
  And I run my ptty command
  Then the ptty exit status should be 0
  And the ptty output should contain "This file is valid"

Scenario: When the project has a partial `.delivery/project.toml`
	  it should fail to execute the NOT configured phase
  When I have a partially config project.toml file
  And I invoke a pseudo tty with command "delivery local unit"
  And I want to debug the pseudo tty command
  And I cd inside my ptty to "local"
  And I run my ptty command
  Then the ptty exit status should be 1
  And the ptty output should contain "Unable to execute an empty phase"

Scenario: When `.delivery/project.toml` file is missing fail and
          show a helpful message about how to recover, additionally
          run the command to prove it will actually fix it
  When I successfully run `rm -rf .delivery/project.toml`
  And I run `delivery local lint`
  Then the exit status should be 1
  And the output should contain:
    """
    The .delivery/project.toml file was not found.

    You can generate this file using the command:
    	chef generate build-cookbook [NAME]
    """
  Then I run `chef generate build-cookbook .`
  And I run `delivery local lint`
  And the exit status should be 0
  And the output should match /Running.*Lint.*Phase/
  And the output should contain "no offenses detected"

Scenario: When local is run with a local project.toml that points to a remote project.toml
  Given I have a remote toml file located at "https://localhost:9999/remote-toml"
  And I have a project.toml with remote_file pointed at "https://localhost:9999/remote-toml"
  When I successfully run `delivery local lint`
  Then the output should contain "REMOTE-LINT"

Scenario: When local is run with a remote toml flag
  Given I have a remote toml file located at "https://localhost:9999/remote-toml"
  When I successfully run `delivery local -r https://localhost:9999/remote-toml lint`
  Then the output should contain "REMOTE-LINT"

Scenario: When local is run with a remote toml flag with erroneous url
  When I invoke a pseudo tty with command "RUST_LOG=debug delivery local -r dont-exist.example.com lint"
  And I want to debug the pseudo tty command
  And I run my ptty command
  Then the ptty exit status should be 1
  And the ptty output should contain "An HTTP Error occurred"

Scenario: Executing the Verify Stage that includes the lint,
          unit and syntax phases.
  When I have a custom project.toml file
  And I run `delivery local verify`
  Then the output should match /Running.*Verify.*Stage/
  And the output should match /Running.*Lint.*Phase/
  And the output should contain "no offenses detected"
  And the output should match /Running.*Syntax.*Phase/
  And the output should match /Running.*Unit.*Phase/
  And the output should contain "This is a cool unit test"
  And the exit status should be 0

Scenario: Executing the Acceptance Stage that includes the
          provision, deploy, smoke and functional phases.
  When I have a custom project.toml file
  And I run `delivery local acceptance`
  Then the output should match /Running.*Acceptance.*Stage/
  And the output should match /Running.*Provision.*Phase/
  And the output should contain "Creating instances"
  And the output should match /Running.*Deploy.*Phase/
  And the output should contain "Converging instances"
  And the output should match /Running.*Smoke.*Phase/
  And the output should contain "Smoking tests"
  And the output should match /Running.*Functional.*Phase/
  And the output should contain "Functional tests"
  And the output should match /Running.*Cleanup.*Phase/
  And the output should contain "Cleaning up"
  And the exit status should be 0

Scenario: Executing All phases at once.
  When I have a custom project.toml file
  And I run `delivery local all`
  Then the output should match /Running.*Every.*Stage/
  And the output should match /Running.*Lint.*Phase/
  And the output should contain "no offenses detected"
  And the output should match /Running.*Syntax.*Phase/
  And the output should match /Running.*Unit.*Phase/
  And the output should contain "This is a cool unit test"
  And the output should match /Running.*Provision.*Phase/
  And the output should contain "Creating instances"
  And the output should match /Running.*Deploy.*Phase/
  And the output should contain "Converging instances"
  And the output should match /Running.*Smoke.*Phase/
  And the output should contain "Smoking tests"
  And the output should match /Running.*Functional.*Phase/
  And the output should contain "Functional tests"
  And the output should match /Running.*Cleanup.*Phase/
  And the output should contain "Cleaning up"
  And the exit status should be 0

Scenario: Executing stages that fails should stop on the failed
          phase and print the exit code.
  When I have a custom project.toml file with failures
  And I run `delivery local verify`
  Then the output should match /Running.*Verify.*Stage/
  And the output should not match /Running.*Syntax.*Phase/
  And the output should not match /Running.*Unit.*Phase/
  And the output should match /Running.*Lint.*Phase/
  And the output should contain "Phase failed with exit code (2)"
  And the exit status should be 1
