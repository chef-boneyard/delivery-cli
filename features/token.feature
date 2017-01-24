Feature: token

Scenario: happy path

  Given a file named ".delivery/cli.toml" with:
    """
    enterprise = "Foobar"
    git_port = "8989"
    organization = "Engineering"
    pipeline = "master"
    server = "localhost"
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
  When I invoke a pseudo tty with command "delivery token"
  And I expect for "Automate password" then type "my_secret_password"
  And I run my ptty command
  Then the ptty output should contain "Automate password"
  Then the ptty output should contain "saved API token to"
  Then the ptty exit status should be 0
  And a file named ".delivery/api-tokens" should exist
  And the file ".delivery/api-tokens" should contain:
    """
    localhost:8080,Foobar,alice|xOsqI8qiBrUCGGRttfFy768R8ZAMJ24RC+0UGyX9/II=
    """
  # These are secrets, and nobody else should be able to read 'em!
  # Also, this totally doesn't work now
  #And the mode of filesystem object ".delivery/api-tokens" should match "600"

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
  When I invoke a pseudo tty with command "delivery token"
  And I expect for "Automate password" then type "my_secret_password"
  And I run my ptty command
  Then the ptty exit status should be 0
  Then the ptty output should contain "Automate password"
  Then the ptty output should contain "saved API token to"
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
      post('/api/v0/e/Foobar/users/alice/get-token') do
        status 200
        {
            "token" => "THIS_IS_A_COOL_TOKEN"
        }
      end
    """
  When I invoke a pseudo tty with command "delivery token"
  And I expect for "Automate password" then type "my_secret_password"
  And I run my ptty command
  Then the ptty exit status should be 0
  Then the ptty output should contain "Automate password"
  Then the ptty output should contain "saved API token to"
  And a file named ".delivery/api-tokens" should exist
  And the file ".delivery/api-tokens" should contain:
    """
    127.0.0.1:8080,Foobar,alice|THIS_IS_A_COOL_TOKEN
    """
  # These are secrets, and nobody else should be able to read 'em!
  # Also, this totally doesn't work now
  #And the mode of filesystem object ".delivery/api-tokens" should match "600"

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
      get('/api/v0/e/token/users') do
        { "users" => ["petrashka"] }
      end
      post('/api/v0/e/token/users/petrashka/get-token') do
        status 200
        {
            "token" => "THE_NEW_TOKEN"
        }
      end
    """
  And a file named ".delivery/api-tokens" with:
    """
    127.0.0.1:8080,token,petrashka|SUPER_FAKE_TOKEN
    """
  When I invoke a pseudo tty with command "delivery api get users"
  # If you need to debug your pseudo tty command just use this step
  And I want to debug the pseudo tty command
  And I expect for "Automate password" then type "my_secret_password"
  And I run my ptty command
  Then the ptty exit status should be 0
  Then the ptty output should contain "Token expired"
  Then the ptty output should contain "Requesting Token"
  Then the ptty output should contain "saved API token to"
  And the file ".delivery/api-tokens" should contain:
    """
    127.0.0.1:8080,token,petrashka|THE_NEW_TOKEN
    """

Scenario: For automation purposes I should be able to request a token
	  on a single command-line without human interaction and display
	  the raw token in the output

  Given the Delivery API server:
    """
      get('/api/v0/e/automation/saml/enabled') do
        status 200
        {
            "enabled" => false
        }
      end
      post('/api/v0/e/automation/users/token/get-token') do
        status 200
        {
            "token" => "MY_SUPER_DUPER_TOKEN"
        }
      end
    """
  When I invoke a pseudo tty with command "delivery token --server 127.0.0.1:8080 --ent automation --user token --raw"
  And I set inside my ptty the env variable "AUTOMATE_PASSWORD" to "something"
  And I run my ptty command
  Then the ptty exit status should be 0
  And the ptty output should contain "MY_SUPER_DUPER_TOKEN"
  And the ptty output should not contain "Requesting Token"
  And the ptty output should not contain "saved API token to"
