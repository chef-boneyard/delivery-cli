Feature: status

Scenario: passing --json when server doesn't know about fips
  Given the Delivery API server on port "9999":
    """
    get('/api/_status') do
      {
        "configuration_mode": "standalone",
        "status": "pong",
        "upstreams": [
          {
            "lsyncd": {
              "status": "not_running"
            },
            "postgres": {
              "status": "pong"
            },
            "rabbitmq": {
              "node_health": {
                "status": "pong"
              },
              "status": "pong",
              "vhost_aliveness": {
                "status": "pong"
              }
            }
          }
        ]
      }
    end
    """
  When I successfully run `delivery status --server=localhost --api-port=9999 --json`
  Then the output should contain:
    """
    {
      "configuration_mode": "standalone",
      "status": "pong",
      "upstreams": [
        {
          "lsyncd": {
            "status": "not_running"
          },
          "postgres": {
            "status": "pong"
          },
          "rabbitmq": {
            "node_health": {
              "status": "pong"
            },
            "status": "pong",
            "vhost_aliveness": {
              "status": "pong"
            }
          }
        }
      ]
    }
    """

Scenario: passing --json when server knows about fips and has bug with configuration_mode flag missing underscore
  Given the Delivery API server on port "9999":
    """
    get('/api/_status') do
      {
        "configuration mode": "standalone",
        "status": "pong",
        "upstreams": [
          {
            "lsyncd": {
              "status": "not_running"
            },
            "postgres": {
              "status": "pong"
            },
            "rabbitmq": {
              "node_health": {
                "status": "pong"
              },
              "status": "pong",
              "vhost_aliveness": {
                "status": "pong"
              }
            }
          }
        ]
      }
    end
    """
  When I successfully run `delivery status --server=localhost --api-port=9999 --json`
  Then the output should contain:
    """
    {
      "configuration_mode": "standalone",
      "status": "pong",
      "upstreams": [
        {
          "lsyncd": {
            "status": "not_running"
          },
          "postgres": {
            "status": "pong"
          },
          "rabbitmq": {
            "node_health": {
              "status": "pong"
            },
            "status": "pong",
            "vhost_aliveness": {
              "status": "pong"
            }
          }
        }
      ]
    }
    """

Scenario: passing --json when server knows about fips
  Given the Delivery API server on port "9999":
    """
    get('/api/_status') do
      {
        "configuration_mode": "standalone",
        "fips_mode": "true",
        "status": "pong",
        "upstreams": [
          {
            "lsyncd": {
              "status": "not_running"
            },
            "postgres": {
              "status": "pong"
            },
            "rabbitmq": {
              "node_health": {
                "status": "pong"
              },
              "status": "pong",
              "vhost_aliveness": {
                "status": "pong"
              }
            }
          }
        ]
      }
    end
    """
  When I successfully run `delivery status --server=localhost --api-port=9999 --json`
  Then the output should contain:
    """
    {
      "configuration_mode": "standalone",
      "fips_mode": "true",
      "status": "pong",
      "upstreams": [
        {
          "lsyncd": {
            "status": "not_running"
          },
          "postgres": {
            "status": "pong"
          },
          "rabbitmq": {
            "node_health": {
              "status": "pong"
            },
            "status": "pong",
            "vhost_aliveness": {
              "status": "pong"
            }
          }
        }
      ]
    }
    """

Scenario: when server knows about fips but it is disabled
  Given the Delivery API server on port "9999":
    """
    get('/api/_status') do
      {
        "configuration_mode": "standalone",
        "fips_mode": "false",
        "status": "pong",
        "upstreams": [
          {
            "lsyncd": {
              "status": "not_running"
            },
            "postgres": {
              "status": "pong"
            },
            "rabbitmq": {
              "node_health": {
                "status": "pong"
              },
              "status": "pong",
              "vhost_aliveness": {
                "status": "pong"
              }
            }
          }
        ]
      }
    end
    """
  When I successfully run `delivery status --server=localhost --api-port=9999 --no-color`
  Then the output should match:
  """
  Status information for Automate server localhost:9999...

  Status: up \(\d+ ms\)
  Configuration Mode: standalone
  FIPS Mode: disabled
  Upstreams:
    Lsyncd:
      status: not_running
    PostgreSQL:
      status: up
    RabbitMQ:
      status: up
      node_health:
        status: up
      vhost_aliveness:
        status: up
  """

Scenario: when server knows about fips but it is enabled
  Given the Delivery API server on port "9999":
    """
    get('/api/_status') do
      {
        "configuration_mode": "standalone",
        "fips_mode": "true",
        "status": "pong",
        "upstreams": [
          {
            "lsyncd": {
              "status": "not_running"
            },
            "postgres": {
              "status": "pong"
            },
            "rabbitmq": {
              "node_health": {
                "status": "pong"
              },
              "status": "pong",
              "vhost_aliveness": {
                "status": "pong"
              }
            }
          }
        ]
      }
    end
    """
  When I successfully run `delivery status --server=localhost --api-port=9999 --no-color`
  Then the output should match:
  """
  Status information for Automate server localhost:9999...

  Status: up \(\d+ ms\)
  Configuration Mode: standalone
  FIPS Mode: enabled
  Upstreams:
    Lsyncd:
      status: not_running
    PostgreSQL:
      status: up
    RabbitMQ:
      status: up
      node_health:
        status: up
      vhost_aliveness:
        status: up

  Your Automate Server is configured in FIPS mode.
  Please add the following to your cli.toml to enable Automate FIPS mode on your machine:

  fips = true
  fips_git_port = OPEN_PORT

  Replace OPEN_PORT with any port that is free on your machine.
  """

Scenario: when server doesn't know about fips
  Given the Delivery API server on port "9999":
    """
    get('/api/_status') do
      {
        "configuration_mode": "standalone",
        "status": "pong",
        "upstreams": [
          {
            "lsyncd": {
              "status": "not_running"
            },
            "postgres": {
              "status": "pong"
            },
            "rabbitmq": {
              "node_health": {
                "status": "pong"
              },
              "status": "pong",
              "vhost_aliveness": {
                "status": "pong"
              }
            }
          }
        ]
      }
    end
    """
  When I successfully run `delivery status --server=localhost --api-port=9999 --no-color`
  Then the output should match:
  """
  Status information for Automate server localhost:9999...

  Status: up \(\d+ ms\)
  Configuration Mode: standalone
  Upstreams:
    Lsyncd:
      status: not_running
    PostgreSQL:
      status: up
    RabbitMQ:
      status: up
      node_health:
        status: up
      vhost_aliveness:
        status: up
  """

Scenario: when server doesn't know about fips and rabbit doesn't return optional content
  Given the Delivery API server on port "9999":
    """
    get('/api/_status') do
      {
        "configuration_mode": "standalone",
        "status": "pong",
        "upstreams": [
          {
            "lsyncd": {
              "status": "not_running"
            },
            "postgres": {
              "status": "pong"
            },
            "rabbitmq": {
              "status": "pong"
            }
          }
        ]
      }
    end
    """
  When I successfully run `delivery status --server=localhost --api-port=9999 --no-color`
  Then the output should match:
  """
  Status information for Automate server localhost:9999...

  Status: up \(\d+ ms\)
  Configuration Mode: standalone
  Upstreams:
    Lsyncd:
      status: not_running
    PostgreSQL:
      status: up
    RabbitMQ:
      status: up
  """
