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

Scenario: Without a token
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
  When I run `delivery api get 'orgs' --server=127.0.0.1 --ent=bar --user=cukes`
  Then the exit status should be 1
  And the output should contain "Missing API token"
    # """
    # Missing API token. Try `delivery token` to create one
    # server: 127.0.0.1, ent: bar, user: cukes
    # """
