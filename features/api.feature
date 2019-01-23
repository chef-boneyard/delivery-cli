Feature: api
@api
Scenario: make a basic call
  Given a dummy api-tokens file
  And a dummy Delivery API server
  When I successfully run `delivery api get 'orgs' --server=localhost --api-port=8080 --ent=dummy --user=link`
  Then the output should contain:
    """
      "orgs": [
        "dummy"
      ]
    """

Scenario: make a basic call with a2_mode
  Given a dummy api-tokens file
  And a dummy A2 Workflow API server
  When I successfully run `delivery api get 'orgs' --server=localhost --api-port=8080 --ent=dummy --user=link --a2-mode`
  Then the output should contain:
    """
      "orgs": [
        "dummy"
      ]
    """

Scenario: Submitting a POST request with data
  Given a dummy api-tokens file
  And a dummy Delivery API server
  When I successfully run `delivery api post 'orgs' -s=localhost --api-port=8080 -e=dummy -u=link -d '{"name":"new_org"}'`
  Then the exit status should be 0

Scenario: Submitting a DELETE request
  Given a dummy api-tokens file
  And I have a dummy cli.toml file
  And a dummy Delivery API server
  When I run `delivery api delete orgs/ganondorf`
  Then the exit status should be 0

Scenario: Without a token and non_interactive enabled
  Given a dummy Delivery API server
  Then a file named ".delivery/cli.toml" with:
    """
      non_interactive = true
    """
  When I run `delivery api get 'orgs' --server=localhost --ent=dummy --user=link`
  Then the exit status should be 1
  And the output should contain "Unable to request token due to --no-interactive flag."
  And the output should contain:
  """
  Missing API token. Try `delivery token` to create one
  """

Scenario: Without a token will first request one.
  Given a dummy Delivery API server
  When I invoke a pseudo tty with command "delivery api get orgs -s=localhost:8080 -e=dummy -u=link"
  And I expect for "Automate password" then type "my_secret_password"
  And I run my ptty command
  Then the ptty exit status should be 0
  And the ptty output should contain "Requesting Token"
  And the ptty output should contain "saved API token to"
  And the ptty output should contain "\"dummy\""
  And the file ".delivery/api-tokens" should contain:
    """
    localhost:8080,dummy,link|xOsqI8qiBrUCGGRttfFy768R8ZAMJ24RC+0UGyX9/II=
    """
    
Scenario: Hitting an invalid endpoint
  Given a dummy api-tokens file
  And I have a dummy cli.toml file
  And a dummy Delivery API server
  When I run `delivery api get not_found`
  Then the exit status should be 1
  And the output should contain "404: Endpoint not found!"
  And the output should contain:
  """
  Unable to access endpoint: https://localhost:8080/api/v0/e/dummy/not_found
  """

Scenario: Submitting a POST request to create an artifact that already exists
  Given a dummy api-tokens file
  And I have a dummy cli.toml file
  And a dummy Delivery API server
  When I run `delivery api post orgs -d '{"name":"existing_org"}'`
  Then the exit status should be 1
  And the output should contain "409 Conflict"

Scenario: Submitting an unknown request
  Given a dummy api-tokens file
  And I have a dummy cli.toml file
  And a dummy Delivery API server
  When I run `delivery api get endpoint_with_unknown_status_code`
  Then the exit status should be 1
  And the output should contain "An API Error occurred"
  And the output should contain "Request returned: '429 Too Many Requests'"

