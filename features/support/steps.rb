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

Given(/^I have a feature branch "(.*)" off of "(.*)"$/) do |branch, base|
  step %(I successfully run `git checkout -b #{branch} #{base}`)
  step "I set the environment variables to:", table(%{
        | variable    |   value   |
        | FAKE_BRANCH | #{branch} |
  })
  step %(I make a commit with message "Add tests first")
  step %(I make a commit with message "Add implementation")
end

# When in a git repository, checks out the given branch. The branch
# must already exist
When(/^I checkout the "(.*?)" branch$/) do |branch|
  in_current_dir do
    step "I set the environment variables to:", table(%{
          | variable    |   value   |
          | FAKE_BRANCH | #{branch} |
    })
    step %(I successfully run `git checkout #{branch}`)
  end
end

Then(/^(["'])([^\1]*)\1 should be run$/) do |_quote, cmd_template|
  cmd = ERB.new(cmd_template).result
  assert_command_run(cmd)
end

Then(/^(["'])([^\1]*)\1 should not be run$/) do |_quote, pattern|
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

Given(/^a user creates a delivery backed project$/) do
  pending "not implemented"
end
Given(/^a user creates a github backed project$/) do
  pending "not implemented"
end
Given(/^a user creates a bitbucket backed project$/) do
  pending "not implemented"
end
Given(/^a bitbucket project is created in delivery$/) do
  pending "not implemented"
end
Given(/^a github project is created in delivery$/) do
  pending "not implemented"
end
Given(/^a change configuring delivery is created$/) do
  pending "not implemented"
end
Given(/^the change has the default generated build_cookbook$/) do
  step %("git push --porcelain --progress --verbose delivery foo:_for/master/add-delivery-config" should be run)
end
