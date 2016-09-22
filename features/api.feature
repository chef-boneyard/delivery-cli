Feature: api

Scenario: make a basic call
  Given a file named ".delivery/api-tokens" with:
    """
    localhost:9999,bar,cukes|this_is_a_fake_token
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
  When I successfully run `delivery api get 'orgs' --server=localhost --api-port=9999 --ent=bar --user=cukes`
  Then the output should contain:
    """
      "orgs": []
    """

Scenario: Submitting a POST request with data
  Given a file named ".delivery/api-tokens" with:
  """
  localhost:9999,bar,cukes|this_is_a_fake_token
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
  When I successfully run `delivery api post 'orgs' -s=localhost --api-port=9999 -e=bar -u=cukes -d '{"name":"dummy"}'`
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
  Then a file named ".delivery/cli.toml" with:
    """
      non_interactive = true
    """
  When I run `delivery api get 'orgs' --server=localhost --ent=bar --user=cukes`
  Then the exit status should be 1
  And the output should contain "Missing API token"
    # """
    # Missing API token. Try `delivery token` to create one
    # server: localhost, ent: bar, user: cukes
    # """

Scenario: Without a token will first request one.

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
        "orgs" => [ "cool_organization" ]
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
  When I invoke a pseudo tty with command "delivery api get orgs -s=localhost:8080 -e=bar -u=cukes"
  And I expect for "Automate password" then type "my_secret_password"
  And I run my ptty command
  Then the ptty exit status should be 0
  Then the ptty output should contain "Requesting Token"
  And the ptty output should contain "saved API token to"
  And the ptty output should contain "cool_organization"
  And the ptty output should contain "saved API token to"
  And the file ".delivery/api-tokens" should contain:
    """
    localhost:8080,bar,cukes|xOsqI8qiBrUCGGRttfFy768R8ZAMJ24RC+0UGyX9/II=
    """
    
