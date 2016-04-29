# Mock a custom config.json
def custom_config
<<EOF
{
  "version": "2",
  "build_cookbook": {
    "path": ".delivery/build-cookbook",
    "name": "build-cookbook"
  },
  "skip_phases": [ "smoke", "security", "syntax", "uni", "quality" ],
  "build_nodes": {},
  "delivery-truck": {
    "publish": {
      "chef_server": true
    }
  },
  "dependencies": []
}
EOF
end

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

Given(/^a dummy Delivery API server/) do
  endpoints = %(
    get('/api/v0/e/dummy/scm-providers') do
      status 200
      [
        {
          "name" => "Github",
          "projectCreateUri" => "/github-projects",
          "scmSetupConfigs" => [
            true
          ],
          "type" => "github",
          "verify_ssl" => true
        },
        {
          "name" => "Bitbucket",
          "projectCreateUri" => "/bitbucket-projects",
          "scmSetupConfigs" => [
            {
              "_links" => {
                "self" => {
                  "href" => "https://127.0.0.1/api/v0/e/skinkworks/bitbucket-servers/dummy.bitbucket.com"
                }
              },
              "root_api_url" => "https://dummy.bitbucket.com",
              "user_id" => "dummy"
            }
          ],
          "type" => "bitbucket"
        }
      ]
    end
    get('/api/v0/e/dummy/orgs/dummy/projects/delivery-cli-init') do
      status 201
      { "error" => "not_found" }
    end
    post('/api/v0/e/dummy/orgs/dummy/projects') do
      status 201
      { "project" => "delivery-cli-init" }
    end
    post('/api/v0/e/dummy/orgs/dummy/projects/delivery-cli-init/pipelines') do
      status 201
      { "pipeline" => "master" }
    end
    post('/api/v0/e/dummy/orgs/dummy/bitbucket-projects') do
      status 201
      { "project" => "delivery-cli-init" }
    end
    post('/api/v0/e/dummy/orgs/dummy/github-projects') do
      status 201
      { "project" => "delivery-cli-init" }
    end
  )
  step %(the Delivery API server on port "8080":), endpoints
end

Given(/^a user creates a delivery backed project$/) do
  step %(I checkout the "add-delivery-config" branch)
  step %(I successfully run `delivery init`)
end

Given(/^a user creates a github backed project$/) do
  step %(I checkout the "add-delivery-config" branch)
  step %(I successfully run `delivery init --github chef --repo-name delivery-cli-init`)
end

Given(/^a user creates a bitbucket backed project$/) do
  step %(I checkout the "add-delivery-config" branch)
  step %(I successfully run `delivery init --bitbucket chef --repo-name delivery-cli-init`)
end

Given(/^a delivery project is created in delivery$/) do
  step %(the output should match /Creating(.*)delivery(.*)project/)
  step %(the output should contain "Remote 'delivery' added to git config")
  step %(the output should contain "Create and checkout add-delivery-config feature branch")
  step %("git push --porcelain --progress --verbose delivery add-delivery-config:_for/master/add-delivery-config" should be run)
end

Given(/^a bitbucket project is created in delivery$/) do
  step %(the output should match /Creating(.*)bitbucket(.*)project/)
  step %(the output should contain "Remote 'delivery' added to git config")
  step %(the output should contain "Create and checkout add-delivery-config feature branch")
  step %("git push --porcelain --progress --verbose delivery add-delivery-config:_for/master/add-delivery-config" should be run)
end

Given(/^a github project is created in delivery$/) do
  step %(the output should match /Creating(.*)github(.*)project/)
  step %(the output should not contain "Remote 'delivery' added to git config")
  step %(the output should contain "Create and checkout add-delivery-config feature branch")
end

Given(/^a change configuring delivery is created$/) do
  step %("git checkout -b add-delivery-config" should be run)
  step %("git commit -m Add Delivery config" should be run)
  step %(the file ".delivery/config.json" should contain:), %("version": "2",)
  step %(the file ".delivery/config.json" should contain:), %("build_cookbook": {)
  step %(the file ".delivery/config.json" should contain:), %("path": ".delivery/build-cookbook")
  step %(the file ".delivery/config.json" should contain:), %("name": "build-cookbook")
  step %(the file ".delivery/config.json" should contain:), %(},)
  step %(the file ".delivery/config.json" should contain:), %("skip_phases": [],)
  step %(the file ".delivery/config.json" should contain:), %("build_nodes": {},)
  step %(the file ".delivery/config.json" should contain:), %("dependencies": [])
end

Given(/^a user creates a project with a custom config\.json$/) do
  step %(a file named "../my_custom_config.json" with:), custom_config
  step %(I checkout the "add-delivery-config" branch)
  step %(I successfully run `delivery init -c ../my_custom_config.json`)
end

Given(/^a change configuring a custom delivery is created$/) do
  step %("git checkout -b add-delivery-config" should be run)
  step %("git commit -m Add Delivery config" should be run)
  step %(the file ".delivery/config.json" should contain exactly:), custom_config
end

Given(/^the change has the default generated build_cookbook$/) do
  step %("git commit -m Add Delivery build cookbook" should be run)
  step %("chef generate cookbook .delivery/build-cookbook" should be run)
  step %(a directory named ".delivery/build-cookbook" should exist)
end

Given(/^specifies the option "([^"]*)"$/) do |option|
  pending "not implemented"
end

Then(/^no build\-cookbook is generated$/) do
  pending "not implemented"
end
