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
  Then the output should match /Status information for Automate server localhost:9999...\n\nStatus: up \(\d+ ms\)\nConfiguration Mode: standalone\nFIPS Mode: disabled\nUpstreams:\n  Lsyncd:\n    status: not_running\n  PostgreSQL:\n    status:up\n  RabbitMQ:\n    status: up\n    node_health:\n      status: up\n    vhost_aliveness:\n      status: up/

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
  Then the output should match /Status information for Automate server localhost:9999...\n\nStatus: up \(\d+ ms\)\nConfiguration Mode: standalone\nFIPS Mode: enabled\nUpstreams:\n  Lsyncd:\n    status: not_running\n  PostgreSQL:\n    status:up\n  RabbitMQ:\n    status: up\n    node_health:\n      status: up\n    vhost_aliveness:\n      status: up\n\nYour Automate Server is configured in FIPS mode.\nPlease add the following to your cli.toml to enable Automate FIPS mode on your machine:\n\nfips = true\nfips_git_port = OPEN_PORT\n\nReplace OPEN_PORT with any port that is free on your machine./

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
  Then the output should match /Status information for Automate server localhost:9999...\n\nStatus: up \(\d+ ms\)\nConfiguration Mode: standalone\nUpstreams:\n  Lsyncd:\n    status: not_running\n  PostgreSQL:\n    status:up\n  RabbitMQ:\n    status: up\n    node_health:\n      status: up\n    vhost_aliveness:\n      status: up/

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
  Then the output should match /Status information for Automate server localhost:9999...\n\nStatus: up \(\d+ ms\)\nConfiguration Mode: standalone\nUpstreams:\n  Lsyncd:\n    status: not_running\n  PostgreSQL:\n    status:up\n  RabbitMQ:\n    status: up/
