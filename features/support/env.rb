require 'aruba/cucumber'
require_relative "../support/api_server"
require 'aruba/platform'

Before do
  @aruba_timeout_seconds = 30
  @fake_bins_history_file = File.join(Aruba.config.home_directory, '.history')
  FileUtils.rm_rf @fake_bins_history_file
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
    histfile = @fake_bins_history_file
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
