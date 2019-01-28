require 'pty'
require 'expect'
require 'tempfile'

module Delivery

  class PtySpawn
    attr_accessor :command, :input, :output_file, :pwd, :environment,
                  :debug

    attr_reader   :exitstatus, :output
    # Execute a command on a pseudo tty 
    #
    # This class is designed to be used in Cucumber tests when the
    # interactive native step doesn't work since we depend on a full
    # tty terminal. It allows you to run a command passing a few args
    # that will enabled you to execute the command in the context of
    # cucumber. Additionally there is a `@debug` arg that when it is
    # enabled you will see everything that is happening with the
    # setup and execution of your command.
    #
    # The implementation allows you to control the behavior of the
    # command you want to execute by configuring @input's with an
    # expected text that when it is found, it will inject something
    # to the tty simulating the actual typing of a user.
    #
    # You can spawn a command on a pseudo tty as follows:
    #
    # @example Hello World
    #   list_tty = Delivery::PtySpawn.new('tty')
    #   list_tty.run
    #   list_tty.output     # Should contain the spawn pseudo tty => ["/dev/ttys002\r\n"]
    #   list_tty.exitstatus # Command exit status => 0
    #
    # @example More advance
    #
    #   cmd = Delivery::PtySpawn.new('delivery command -with options', {
    #           "pwd" => "/my/custom/path",
    #           "environment" => { "PATH" => "Modify:My:PATH:Env:Variable" },
    #           "output_file" => "/my/output/file.out",
    #           "debug" => true
    #         })
    #   cmd.expect_and_type("a specific text", "send something cool")
    #   cmd.expect_and_type("wait for the next text", "input_input_input")
    #   cmd.run
    #   cmd.exitstatus              # The exitstatus of the command we ran
    #   cmd.write_output_to_file
    #   => Writing output to: /my/output/file.out
    #
    # @param command [String] The command we will execute in the pseudo tty
    # @param opts [Hash] Extra options like: `pwd`, `environment`, `output_file`
    # @return [Delivery::PtySpawn]
    def initialize(command, opts = {})
      @command = command
      @pwd = opts["pwd"] || nil
      @environment = opts["environment"] || {}
      @output_file = opts["output_file"] || Tempfile.new('cmd_ptty').path
      @debug = opts['debug'] || false
      @input = opts['input'] || {}
      @output = []
    end

    # Store a new input for the command you will execute
    #
    # Every input has an `expect` and `type` variables and you can save
    # as many as you want. These inputs works as follow:
    #
    # When we run the command
    #  It will expect some specific text
    #  Then it will simulate that a user type something to the tty
    #
    # @param expect [String] The text we will wait to find when executing the command
    # @param type [String] the input value we want to send when we find the text we expect
    def expect_and_type(expect, type)
      puts "Adding input: { '#{expect}' => '#{type}' }" if @debug
      @input["#{expect}"] = type
    end

    # Add an environment variable to the ptty
    #
    # @param name  [String] Then name of the environment variable to add
    # @param value [String] The value of the environment variable to add
    def add_env_variable(name, value)
      puts "Adding env_variable: { '#{name}' => '#{value}' }" if @debug
      @environment["#{name}"] = value
    end

    # Cd into a directory
    def cd(dir)
      @pwd = File.join(@pwd, dir)
    end

    # Convert the output to String
    def output_str
      @output.join('')
    end

    # Write the output to a file
    def write_output_to_file
      puts "Writing output to: #{@output_file}" if @debug
      f = open(@output_file, 'w+')
      f.write(output_str)
      f.close
    end

    # Generate the command we will run
    #
    # This will add any environment variables we have configured as
    # well as changing the directory where we want the command to be
    # executed.
    def command
      cmd = ""
      @environment.each do |name, value|
        cmd += "export #{name}=#{value};"
      end
      cmd += "cd #{@pwd};" if @pwd
      cmd += @command
    end

    # Execute the command on a pseudo tty
    #
    # This is the actual execution of the command, it will inject the
    # @input's we have so that we can control its behavior. Once the
    # process has finished we store its exit status and output
    def run
      puts "Running pseudo tty command: #{command}" if @debug
      puts "Output:" if @debug

      PTY.spawn(command) do |stdin, stdout, pid|
        begin
          # Go through all the inputs we saved
          @input.each do |expect_str, type_str|
            # Expect a string
            out = stdin.expect(expect_str)
            # Inject a text as if someone typed it
            stdout.puts(type_str)
            @output << out
            puts out if @debug
          end

          # Continue reading the output until EOF is found
          until stdin.eof? do
            out = stdin.readline
            @output << out
            puts out if @debug
          end
        rescue Errno::EIO
          puts "Errno:EIO that probably means the child process finished " +
               "and closed the stream" if @debug
        ensure
          Process.wait pid
        end
      end

      # Store the exitstatus of the spawn process
      @exitstatus = $?.exitstatus
    end
  end
end
