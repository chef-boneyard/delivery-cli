require 'rack'
require 'grape'
require 'openssl'
require 'webrick/https'

module Delivery

  class StubAPI

    # Create a new Delivery API server implementing the code in
    # `block`.
    #
    # This is intended for use in Cucumber tests, where we may only
    # need a small subset of the Delivery API to test a particular
    # feature. Additionally, we may want tailor-made interactions for
    # specific tests; defining all this in one master server would
    # become cumbersome. This way, the server's behavior can be
    # fine-tuned in the tests themselves.
    #
    # The block can contain anything you'd add to a `Grape::API`
    # subclass (see http://intridea.github.io/grape/ for more). As you
    # can see in the example, it's a pretty straightforward DSL. You
    # can customize which endpoints the server responds to, as well as
    # the port.
    #
    # You can create and manage the server on your own (without using
    # this method) if you really want to, but why bother?
    #
    # @example Hello World
    #   Delivery::StubAPI.start_server(8080) do
    #     get(/hello-world) do
    #       {"message" => "hello world"}
    #     end
    #   end
    #
    # @param port [Fixnum] The port on which the server listens
    # @return [Delivery::StubAPI]
    def self.start_server(port, &block)

      # Create a new anonymous subclass of the Grape API,
      # pre-configuring it to be a pure JSON API
      klass = Class.new(Grape::API) do
        content_type :txt, 'text/plain'
        content_type :json, 'application/json'
        format :json
        default_format :json
      end

      # This is where we dynamically set up the endpoint behavior
      klass.class_eval(&block)

      # FIRE IT UP!
      server = new(klass.new)
      server.start(port)
      server
    end

    # @param app [#call] A Rack application object containing logic
    #   for the server
    #
    # @return [Delivery::StubAPI]
    def initialize(app)
      @app = app
      @server = nil # WEBrick::HTTPServer
    end

    # Starts the configured Rack application server
    #
    # @param port [Fixnum] The port on which the server listens
    # @return [void]
    def start(port)
      Rack::Handler::WEBrick.run(@app, config(port)){|s| @server = s }
    end

    # WEBrick configuration to be used in the server.
    #
    # A few things to note:
    # - All logging is disabled, so as not to pollute the Cucumber
    #   output. If you need to debug failing tests, consider
    #   commenting out the `:Logger` and `:AccessLog`
    #   configurations. See `WEBrick::Log`, `WEBrick::BasicLog`, and
    #   `WEBrick::AccessLog` for more
    # - The server runs in its own thread
    # - The server serves HTTPS on the assigned `port` using a self-signed
    #   certificate, which WEBrick helpfully creates when you supply a
    #   `:SSLCertName`.
    #
    # @param port [Fixnum] The port on which the server listens
    # @return [Hash]
    def config(port)
      {
        :Logger                     => WEBrick::Log::new(nil, 0),
        :AccessLog                  => [],
        :ServerType                 => Thread,
        :ShutdownSocketWithoutClose => true,
        :Port                       => port,
        :SSLEnable                  => true,
        :SSLVerifyClient            => OpenSSL::SSL::VERIFY_NONE,
        :SSLCertName                => [["CN", "delivery-cli-test"]]
      }
    end

    # Stop the Rack application server
    #
    # @return [void]
    def stop
      @server.shutdown
    end

  end
end
