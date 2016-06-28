require 'aruba/cucumber'
require_relative "../support/api_server"

# Path for fake binaries so we can test things like git
# interaction. We're going to stick this early on the path so the CLI
# will hit it first.
fake_bin = File.expand_path('../fakebin', __FILE__)

# Be able to call original binary in mocks.
system_git = `which git 2>/dev/null`.chomp
system_cp = `which cp 2>/dev/null`.chomp
system_mv = `which mv 2>/dev/null`.chomp
system_chef = `which chef 2>/dev/null`.chomp

# Ensure that the delivery binary we just built is the one that we're
# using.
cli_dir = File.expand_path('../../../target/release', __FILE__)

# Chances are high that we're running the tests from a delivery-cli
# directory that already has a `.delivery/cli.toml` file present,
# which would affect the results of some tests. We can bypass this
# situation by having the tests take place in a random temporary
# directory.
Before do
  @aruba_timeout_seconds = 30

  # Don't use 'tmp/aruba' as the current directory, since it has the
  # potential to confound tests involving delivery CLI configuration
  # file lookups (we search up the directory tree until we find a
  # `.delivery/cli.toml`, and your local config will mess with the
  # test results. Using a temp directory that's well out of the way
  # will alleviate this.
  #
  # We'll also categorically use this directory as $HOME in all tests.
  current_directory = Dir.mktmpdir
  @dirs = [current_directory]
  set_env('HOME', current_directory)
  set_env('EMAIL', 'cukes@mycompany.com')

  set_env('DELIVERY_SYSTEM_GIT', system_git)
  set_env('DELIVERY_SYSTEM_CP', system_cp)
  set_env('DELIVERY_SYSTEM_MV', system_mv)
  set_env('DELIVERY_SYSTEM_CHEF', system_chef)

  set_env('PATH', "#{cli_dir}#{File::PATH_SEPARATOR}#{fake_bin}#{File::PATH_SEPARATOR}#{ENV['PATH']}")
  # We don't use the `HOME` env var for this as it is frequently
  # overriden to other values when running executables
  # (notably when running chef-client in the CLI)
  set_env('FAKE_BINS_HISTORY_FILE', File.join(current_directory, '.history'))

  fixture_repos_dir = File.expand_path("../../fixtures/repos", __FILE__)
  set_env('GIT_CLONE_FIXTURE_REPOS_DIR', fixture_repos_dir)
end

Before('@broken') do
  pending
end

After do
  @server.stop if @server
end

World Module.new {

  # Inspired from hub
  def history
    histfile = ENV['FAKE_BINS_HISTORY_FILE']
    if File.exist? histfile
      File.readlines histfile
    else
      []
    end
  end

  # Stolen from hub
  def assert_command_run cmd
    cmd += "\n" unless cmd[-1..-1] == "\n"
    expect(history).to include(cmd)
  end

}
