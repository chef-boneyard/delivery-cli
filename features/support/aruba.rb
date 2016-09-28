require 'aruba/cucumber'
require_relative "../support/api_server"
require 'aruba/platform'

# Path for fake binaries so we can test things like git
# interaction. We're going to stick this early on the path so the CLI
# will hit it first.
fake_bin = File.expand_path('../fakebin', __FILE__)

# Be able to call original binary in mocks.
system_git = `which git 2>/dev/null`.chomp

# Ensure that the delivery binary we just built is the one that we're
# using.
cli_dir = File.expand_path('../../../target/release', __FILE__)
fixture_repos_dir = File.expand_path("../../fixtures/repos", __FILE__)

# Chances are high that we're running the tests from a delivery-cli
# directory that already has a `.delivery/cli.toml` file present,
# which would affect the results of some tests. We can bypass this
# situation by having the tests take place in a random temporary
# directory.
Aruba.configure do |config|
  # Don't use 'tmp/aruba' as the root directory, since it has the
  # potential to confound tests involving delivery CLI configuration
  # file lookups (we search up the directory tree until we find a
  # `.delivery/cli.toml`, and your local config will mess with the
  # test results. Using a temp directory that's well out of the way
  # will alleviate this.
  #
  # We'll also categorically use this directory as $HOME in all tests.
  config.root_directory     = Dir.mktmpdir
  config.working_directory  = "delivery-cli/"
  config.home_directory     = File.join(config.root_directory, config.working_directory)

  @fake_bins_history_file = File.join(config.home_directory, '.history')

  # We will set the Environment Variable to Aruba.config.home_directory
  # before every test, also we will lay down the .history that will be
  # deleted so it is empty for the current test
  config.command_runtime_environment = {
    'HOME' => config.home_directory,
    'FAKE_BINS_HISTORY_FILE' => @fake_bins_history_file,
    'EMAIL' => 'cukes@mycompany.com',
    'DELIVERY_SYSTEM_GIT' => system_git,
    'GIT_CLONE_FIXTURE_REPOS_DIR' => fixture_repos_dir,
    'PATH' => "#{cli_dir}#{File::PATH_SEPARATOR}#{fake_bin}#{File::PATH_SEPARATOR}#{ENV['PATH']}"
  } 
end
