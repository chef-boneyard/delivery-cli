# Creates a new directory, "git init"s it and creates an empty commit
# so we can have a branch
Given(/^a git repo "(.*?)"$/) do |repo|
  step %(a directory named "#{repo}")
  dirs.push(repo)
  step %(I successfully run `git init --quiet`)
  step %(I make a commit with message "Initial commit")
  dirs.pop
end

Given(/^I make a commit with message "([^"]+)"$/) do |message|
  step %(I successfully run `git commit --quiet -m '#{message}' --allow-empty`)
end

Given(/^I am in the "([^"]*)" git repo$/) do |repo|
  step %(a git repo "#{repo}")
  step %(I cd to "#{repo}")
end

Given(/^I have a feature branch "([^"]*)" off of "([^"]*)"$/) do |branch, base|
  step %(I successfully run `git checkout -b #{branch} #{base}`)
  step %(I make a commit with message "Add tests first")
  step %(I make a commit with message "Add implementation")
end

# When in a git repository, checks out the given branch. The branch
# must already exist
When(/^I checkout the "(.*?)" branch$/) do |branch|
  in_current_dir do
    step %(I successfully run `git checkout #{branch}`)
  end
end

Then(/^"([^"]*)" should be run$/) do |cmd|
  assert_command_run(cmd)
end

Then(/^"([^"]+)" should not be run$/) do |pattern|
  history.each { |h| expect(h).to_not include(pattern) }
end

Given(/^the Delivery API server:$/) do |endpoints|
  step %(the Delivery API server on port "8080":), endpoints
end

Given(/^the Delivery API server on port "(\d+)":$/) do |port, endpoints|
  @server = Delivery::StubAPI.start_server(port) do
    eval(endpoints, binding)
  end
end
