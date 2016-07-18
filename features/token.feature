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
      get('/api/v0/e/Foobar/saml/enabled') do
        status 200
        {
            "enabled" => false
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

@broken
Scenario: SAML enabled API not yet available in Automate (old version)

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
      get('/api/v0/e/Foobar/saml/enabled') do
        status 404
        {
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

Scenario: SAML enabled in Automate

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
      get('/api/v0/e/Foobar/saml/enabled') do
        status 200
        {
            "enabled" => true
        }
      end
      get('/api/v0/e/Foobar/orgs') do
        {
          "_links" => {
            "create_org" => {
              "href" => "/api/v0/e/Foobar/orgs"
            },
            "show_org" => {
              "href" => "/api/v0/e/Foobar/orgs/{org_name}",
              "templated" => true
            }
          },
          "orgs" => []
        }
      end
    """
  When I run `delivery token` interactively
  And I type "Enter"
  And I type "xOsqI8qiBrUCGGRttfFy768R8ZAMJ24RC+0UGyX9/II="
  Then the exit status should be 0
  And the output should contain:
  """
  Press Enter to open a browser window to retrieve a new token.
  """
  And a file named ".delivery/api-tokens" should exist
  And the file ".delivery/api-tokens" should contain:
    """
    127.0.0.1:8080,Foobar,alice|xOsqI8qiBrUCGGRttfFy768R8ZAMJ24RC+0UGyX9/II=
    """

Scenario: SAML overridden (enabled) in config

  Given a file named ".delivery/cli.toml" with:
    """
    enterprise = "Foobar"
    git_port = "8989"
    organization = "Engineering"
    pipeline = "master"
    server = "127.0.0.1"
    api_port = "8080"
    user = "alice"
    saml = true
    """
  And the Delivery API server:
    """
      get('/api/v0/e/Foobar/orgs') do
        {
          "_links" => {
            "create_org" => {
              "href" => "/api/v0/e/Foobar/orgs"
            },
            "show_org" => {
              "href" => "/api/v0/e/Foobar/orgs/{org_name}",
              "templated" => true
            }
          },
          "orgs" => []
        }
      end
    """
  When I run `delivery token` interactively
  And I type "Enter"
  And I type "xOsqI8qiBrUCGGRttfFy768R8ZAMJ24RC+0UGyX9/II="
  Then the exit status should be 0
  And the output should contain:
  """
  Press Enter to open a browser window to retrieve a new token.
  """
  And a file named ".delivery/api-tokens" should exist
  And the file ".delivery/api-tokens" should contain:
    """
    127.0.0.1:8080,Foobar,alice|xOsqI8qiBrUCGGRttfFy768R8ZAMJ24RC+0UGyX9/II=
    """

@broken
Scenario: SAML enabled in Automate but overridden in config

  Given a file named ".delivery/cli.toml" with:
    """
    enterprise = "Foobar"
    git_port = "8989"
    organization = "Engineering"
    pipeline = "master"
    server = "127.0.0.1"
    api_port = "8080"
    user = "alice"
    saml = false
    """
  And the Delivery API server:
    """
      get('/api/v0/e/Foobar/orgs') do
        {
          "_links" => {
            "create_org" => {
              "href" => "/api/v0/e/Foobar/orgs"
            },
            "show_org" => {
              "href" => "/api/v0/e/Foobar/orgs/{org_name}",
              "templated" => true
            }
          },
          "orgs" => []
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

@broken
Scenario: Token expired should trigger an automatic token request

  Given a file named ".delivery/cli.toml" with:
    """
    enterprise = "token"
    git_port = "8989"
    pipeline = "master"
    server = "127.0.0.1"
    api_port = "8080"
    user = "petrashka"
    non_interactive = false
    """
  And the Delivery API server:
    """
      get('/api/v0/e/token/orgs') do
        status 401
        {
          "error"=> "token_expired"
        }
      end
    """
  And a file named ".delivery/api-tokens" with:
    """
    127.0.0.1:8080,Foobar,alice|SUPER_FAKE_TOKEN
    """
  When I run `delivery api get users` interactively
  And I type "my_secret_password"
  Then the exit status should not be 0
  And the output should contain:
  """
  Requesting Token
  """
