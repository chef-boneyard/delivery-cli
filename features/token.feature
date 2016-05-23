Feature: token

@broken
Scenario: happy path

  NOTE: It appears that there's a problem with the way the CLI
  accepts password input. So this doesn't really pass unless you type
  something AS THE TEST RUNS. I think it has to do with stdin maybe
  not being a TTY?

  ALSO NOTE: We could actually do some tests around header values here
  :)

  Given a file named ".delivery/cli.toml" with:
    """
    enterprise = "Foobar"
    git_port = "8989"
    organization = "Engineering"
    pipeline = "master"
    server = "127.0.0.1"
    api_port = "8080"
    user = "alice"
    """
  And the Delivery API server:
    """
      get('/api/v0/e/Foobar/saml/lookup-user/alice') do
        status 200
        {
            "saml_user" => false
        }
      end
      post('/api/v0/e/Foobar/users/alice/get-token') do
        status 200
        {
            "token" => "xOsqI8qiBrUCGGRttfFy768R8ZAMJ24RC+0UGyX9/II="
        }
      end
    """
  When I run `delivery token` interactively
  And I type "my_secret_password"
  Then the exit status should be 0
  And a file named ".delivery/api-tokens" should exist
  And the file ".delivery/api-tokens" should contain:
    """
    127.0.0.1:8080,Foobar,alice|xOsqI8qiBrUCGGRttfFy768R8ZAMJ24RC+0UGyX9/II=
    """
  # These are secrets, and nobody else should be able to read 'em!
  # Also, this totally doesn't work now
  And the mode of filesystem object ".delivery/api-tokens" should match "600"

Scenario: SAML user

  Given a file named ".delivery/cli.toml" with:
    """
    enterprise = "Foobar"
    git_port = "8989"
    organization = "Engineering"
    pipeline = "master"
    server = "127.0.0.1"
    api_port = "8080"
    user = "alice"
    """
  And the Delivery API server:
    """
      get('/api/v0/e/Foobar/saml/lookup-user/alice') do
        status 200
        {
            "saml_user" => true
        }
      end
    """
  When I run `delivery token` interactively
  Then the exit status should be 1
  And the output should contain "Token retrieval for SAML authenticated users is not supported yet."
