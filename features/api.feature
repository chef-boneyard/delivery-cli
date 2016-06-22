Feature: api

Scenario: make a basic call
  Given a file named ".delivery/api-tokens" with:
    """
    127.0.0.1:9999,bar,cukes|this_is_a_fake_token
    """
  And the Delivery API server on port "9999":
    """
    get('/api/v0/e/bar/orgs') do
      {
        "_links" => {
          "create_org" => {
            "href" => "/api/v0/e/bar/orgs"
          },
          "show_org" => {
            "href" => "/api/v0/e/bar/orgs/{org_name}",
            "templated" => true
          }
        },
        "orgs" => []
      }
    end
    """
  When I successfully run `delivery api get 'orgs' --server=127.0.0.1 --api-port=9999 --ent=bar --user=cukes`
  Then the output should contain:
    """
      "orgs": []
    """

Scenario: Submitting a POST request with data
  Given a file named ".delivery/api-tokens" with:
    """
    127.0.0.1:9999,bar,cukes|this_is_a_fake_token
    """
  And the Delivery API server on port "9999":
    """
    get('/api/v0/e/bar/orgs') do
      status 200
      { "orgs" => [] }
    end

    desc 'Create an organization.'
    params do
      requires :name, type: String, desc: 'Org name'
    end
    post '/api/v0/e/bar/orgs' do
      if params[:name] != "dummy"
	status 500
      else
	status 201
      end
    end
    """
  When I successfully run `delivery api post 'orgs' -s=127.0.0.1 --api-port=9999 -e=bar -u=cukes -d '{"name":"dummy"}'`
  Then the exit status should be 0

Scenario: Without a token and non_interactive enabled
  Given the Delivery API server:
    """
    get('/api/v0/e/bar/orgs') do
      {
        "_links" => {
          "create_org" => {
            "href" => "/api/v0/e/bar/orgs"
          },
          "show_org" => {
            "href" => "/api/v0/e/bar/orgs/{org_name}",
            "templated" => true
          }
        },
        "orgs" => []
      }
    end
    """
  And a file named ".delivery/cli.toml" with:
    """
      non_interactive = true
    """
  When I run `delivery api get 'orgs' --server=127.0.0.1 --ent=bar --user=cukes`
  Then the exit status should be 1
  And the output should contain "Missing API token"
    # """
    # Missing API token. Try `delivery token` to create one
    # server: 127.0.0.1, ent: bar, user: cukes
    # """

@broken
Scenario: Without a token will first request one

  NOTE: As we are now quickly prompting for a token when it does not
  exist, this test has the same problem as the `token` one where the CLI
  is asking for a `password` and it doesn't accept it

  Given the Delivery API server:
    """
    get('/api/v0/e/bar/orgs') do
      {
        "_links" => {
          "create_org" => {
            "href" => "/api/v0/e/bar/orgs"
          },
          "show_org" => {
            "href" => "/api/v0/e/bar/orgs/{org_name}",
            "templated" => true
          }
        },
        "orgs" => []
      }
    end
    post('/api/v0/e/bar/users/cukes/get-token') do
      status 200
      {
         "token" => "xOsqI8qiBrUCGGRttfFy768R8ZAMJ24RC+0UGyX9/II=",
         "_links" => {
           "revoke-user-token" => {
             "href" => "/v0/e/bar/users/cukes/revoke-token"
           }
         }
      }
    end
    """
  When I run `delivery api get 'orgs' --server=127.0.0.1 --ent=bar --user=cukes` interactively
  And I type "my_secret_password"
  Then the exit status should be 0
  Then the output should contain:
  """
    Requesting Token
    Delivery password:
    "orgs": []
  """
