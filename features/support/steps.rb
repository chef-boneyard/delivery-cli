require_relative 'pty_spawn'
require_relative 'helpers'
require 'uri'

Given(/^I invoke a pseudo tty with command "(.*)"$/) do |command|
  @current_pty = Delivery::PtySpawn.new(command, {
                  "pwd" => aruba.config.home_directory,
                  "environment" => aruba.config.command_runtime_environment
                 })
end

Given(/^I expect for "(.*)" then type "(.*)"$/) do |exp, txt|
  @current_pty.expect_and_type(exp, txt)
end

Given(/^I set inside my ptty the env variable "(.*)" to "(.*)"$/) do |name, value|
  @current_pty.add_env_variable(name, value)
end

Given(/^I cd inside my ptty to "(.*)"$/) do |dir|
  @current_pty.cd(dir)
end

Given(/^I run my ptty command$/) do
  @current_pty.run
end

Given(/^the ptty output should not contain "(.*)"$/) do |string|
  if @current_pty.output_str.match(/#{string}/)
    raise "The output of the pseudo tty command did match with #{string}" +
          "\nOutput: #{@current_pty.output_str}"
  end
end

Given(/^the ptty output should contain "(.*)"$/) do |string|
  unless @current_pty.output_str.match(/#{string}/)
    raise "The output of the pseudo tty command didn't match with #{string}" +
          "\nOutput: #{@current_pty.output_str}"
  end
end

Given(/^the ptty exit status should be (.*)$/) do |exitstatus|
  unless @current_pty.exitstatus == exitstatus.to_i
    raise "The exit status of the pseudo tty command didn't match: " +
          "#{@current_pty.exitstatus} != #{exitstatus}"
  end
end

Given(/^I want to debug the pseudo tty command$/) do
  @current_pty.debug = true
end

Given(/^I am in a chefdk generated cookbook called "(.*)"$/) do |cb_name|
  step %(I successfully run `chef generate cookbook #{cb_name}`)
  step %(I cd to "#{cb_name}")
end

Given(/^I have an incomplete project.toml file$/) do
  step %(a file named ".delivery/project.toml" with:), incomplete_project_toml
end

Given(/^I have a partially config project.toml file$/) do
  step %(a file named ".delivery/project.toml" with:), partial_project_toml
end

Given(/^I have a custom project.toml file$/) do
  step %(a file named ".delivery/project.toml" with:), project_toml
end

Given(/^I have a custom project.toml file with failures$/) do
  step %(a file named ".delivery/project.toml" with:), project_toml_with_failures
end

Given(/^I have a valid cli.toml file$/) do
  step %(a file named ".delivery/cli.toml" with:), valid_cli_toml
end

Given(/^I have a dummy cli.toml file$/) do
  step %(a file named ".delivery/cli.toml" with:), dummy_cli_toml
end

Given(/^I have a dummy A2 cli.toml file$/) do
  step %(a file named ".delivery/cli.toml" with:), dummy_a2_cli_toml
end

Given(/^a dummy api-tokens file$/) do
  step %(a file named ".delivery/api-tokens" with:),
  """
  localhost:8080,dummy,link|this_is_a_fake_token
  localhost:8080/workflow,dummy,link|this_is_a_fake_token
  """
end

Given(/^I have a valid cli\.toml file with with "([^"]*)":$/) do |append_str|
  step %(a file named ".delivery/cli.toml" with:), valid_cli_toml + "\n" + append_str
end

Given(/^I have a project.toml with remote_file pointed at "([^"]*)"$/) do |url|
  step %(a file named ".delivery/project.toml" with:), project_toml_with_remote_file(url)
end

Given(/^I have a custom generator cookbook with no config generator$/) do
  step %(I have a custom generator cookbook)
  step %(a file named "#{tmp_relative_path}/test-generator/recipes/build_cookbook.rb" with:), build_cookbook_rb
end

Given(/^I have a custom generator cookbook$/) do
  step %(I successfully run `rm -rf #{tmp_relative_path}/test-generator`)
  step %(I successfully run `chef generate generator #{tmp_relative_path}/test-generator`)
  # throw a file into the custom generator we can check for later
  step %(I append to "#{tmp_relative_path}/test-generator/recipes/build_cookbook.rb" with:), additional_gen_recipe
end

# Creates a new directory, "git init"s it and creates an empty commit
# so we can have a branch
Given(/^a git repo "(.*?)"$/) do |repo|
  step %(a directory named "#{repo}")
  step %(I cd to "#{repo}")
  step %(I successfully run `git init --quiet`)
  step %(I make a commit with message "Initial commit")
end

Given(/^I commit all files with message "([^"]+)"$/) do |message|
  step %(I successfully run `git add .`)
  step %(I successfully run `git commit --quiet -m '#{message}'`)
end

Given(/^I make a commit with message "([^"]+)"$/) do |message|
  step %(I successfully run `git commit --quiet -m '#{message}' --allow-empty`)
end

Given(/^I am in the "([^"]*)" git repo$/) do |repo|
  step %(a git repo "#{repo}")
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

Given(/^I set up basic delivery and git configs$/) do
  step %(a file named ".delivery/cli.toml" with:), basic_delivery_config
  step %(a file named ".git/config" with:), basic_git_config
end

Given(/^I already have a .delivery\/config.json on disk$/) do
  step %(a file named ".delivery/config.json" with:), default_delivery_config
end

Given(/^I am in a blank workspace$/) do
  step %(I cd to "..")
  step %(a directory named "workspace")
  step %(I cd to "workspace")
end

Given(/^I clean up the ruby env so I can run other ruby bins like ChefDK$/) do
  step "I set the environment variables to:", table(%{
          | variable        | value |
          | RUBYOPT         |       |
          | BUNDLE_PATH     |       |
          | BUNDLE_BIN_PATH |       |
          | BUNDLE_GEMFILE  |       |
  })
end

Given("I have a repository with failing tests") do
  step("I set the environment variables to:", table(%q[
    | variable             | value          |
    | REPO_TO_COPY         | failing_tests  |
  ]))
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

Given(/^I have a remote toml file located at "([^"]*)"$/) do |url|
  uri = URI(url)
  @server = Delivery::StubAPI.start_server(uri.port) do
    get(uri.path) do
      status 200
      content_type 'text/plain'
      remote_project_toml
    end
  end
end

When(/^I wait for (\d+) seconds?$/) do |n|
    sleep(n.to_i)
end

Given(/^a user creates a delivery backed project$/) do
  step %(I successfully run `delivery init`)
end

Given(/^a user creates a GitHub backed project$/) do
  step %(I successfully run `delivery init --github chef --repo-name delivery-cli-init`)
end

Given(/^a user creates a bitbucket backed project$/) do
  step %(I successfully run `delivery init --bitbucket chef --repo-name delivery-cli-init`)
end

Given(/^a user tries to create a delivery backed project with a custom generator$/) do
  step %(I run `delivery init --generator #{tmp_expanded_path}/test-generator`)
end

Given(/^a user tries to create a delivery backed project with a custom config and custom generator$/) do
  step %(I run `delivery init -c ../my_custom_config.json --generator #{tmp_expanded_path}/test-generator`)
end

Given (/^the delivery remote should exist$/) do
  step %(I successfully run `git config --get remote.delivery.url`)
end

Given(/^a delivery project is created in delivery$/) do
  step %(the output should match /Delivery project named.*was created/)
end

Given(/^a delivery project should not be created in delivery$/) do
  step %(the output should not match /Delivery project named.*was created/)
  step %(the output should match /Delivery project named .* already exists/)
end

Given(/^a bitbucket project is created in delivery$/) do
  step %(the output should match /Bitbucket backed Delivery project named .* created./)
end

Given(/^a GitHub project is created in delivery$/) do
  step %(the output should match /GitHub backed Delivery project named .* created./)
end

Given(/^a default config.json is created$/) do
  step %(the file ".delivery/config.json" should contain:), %("version": "2",)
  step %(the file ".delivery/config.json" should contain:), %("build_cookbook": {)
  step %(the file ".delivery/config.json" should contain:), %("path": ".delivery/build_cookbook")
  step %(the file ".delivery/config.json" should contain:), %("name": "build_cookbook")
  step %(the file ".delivery/config.json" should contain:), %(},)
  step %(the file ".delivery/config.json" should contain:), %("skip_phases": [],)
  step %(the file ".delivery/config.json" should contain:), %("job_dispatch": {)
  step %(the file ".delivery/config.json" should contain:), %("version": "v2")
  step %(the file ".delivery/config.json" should contain:), %("dependencies": [])
end

Given(/^a change to the delivery config is not committed$/) do
  step %("git commit -m Adds Delivery config" should not be run)
end

Given(/^a user creates a project with a custom config\.json$/) do
  step %(a custom config)
  step %(I successfully run `delivery init -c ../my_custom_config.json`)
end

Given(/^a user creates a project with both a custom generator and custom config$/) do
  step %(I have a custom generator cookbook)
  step %(a custom config)
  step %(I successfully run `delivery init -c ../my_custom_config.json --generator #{tmp_expanded_path}/test-generator`)
end

Given(/^a custom config$/) do
  step %(a file named "../my_custom_config.json" with:), custom_config
end

Given(/^I have a config where the build_cookbook comes from Supermarket$/) do
  step %(a file named ".delivery/config.json" with:), config_build_cookbook_from_supermarket
end

Given(/^I have already a custom config$/) do
  step %(a file named ".delivery/config.json" with:), custom_config
end

Given(/^I have already a custom config with a custom build_cookbook path$/) do
  step %(a file named ".delivery/config.json" with:), config_with_custom_build_cookbook
end

Given(/^a change configuring a custom delivery is created$/) do
  step %("git checkout -b add-delivery-config" should be run)
  step %("git commit -m Adds custom Delivery config" should be run)
  step %(the file ".delivery/config.json" should contain exactly:), custom_config
end

Given(/^the change has the default generated build_cookbook$/) do
  step %(the change has a generated build_cookbook called ".delivery/build_cookbook")
end

Given(/^the change has a generated build_cookbook called "([^"]*)"$/) do |bk_path|
  step %("git push --set-upstream --porcelain --progress --verbose delivery master" should be run)
  step %("git commit -m Add generated delivery configuration" should be run)
  step %("git commit -m Add generated delivery build cookbook" should be run)
  step %(a directory named "#{bk_path}" should exist)
end

Given(/^the change does not have the default generated build_cookbook$/) do
  step %("git commit -m Adds Delivery build cookbook and config" should not be run)
  step %("chef generate cookbook .delivery/build_cookbook" should not be run)
  step %(the file ".delivery/build_cookbook" should not exist)
end

Given(/^a user creates a delivery backed project with option "([^"]*)"$/) do |option|
  step %(I successfully run `delivery init #{option}`)
end

Given(/^a generator cookbook cache exists$/) do
  step %(a directory named ".delivery")
  step %(a directory named ".delivery/cache")
  step %(a directory named ".delivery/cache/generator-cookbooks")
end

Given(/^a custom build cookbook is already downloaded in the cache$/) do
  step %(I successfully run `chef generate generator ../.delivery/cache/generator-cookbooks/test-generator`)
  step %(I append to "../.delivery/cache/generator-cookbooks/test-generator/recipes/build_cookbook.rb" with:), additional_gen_recipe
end

Given(/^a custom config is generated$/) do
  step %(the output should match /Custom Delivery config copied from .* to .*/)
  step %(the output should contain "Custom delivery config committed to feature branch.")
  step %("git commit -m Adds custom Delivery config" should be run)
end

Given(/^a custom build_cookbook is generated from "([^"]*)"$/) do |type|
  case type
  when "local_path"
    step %(the output should match /Copying custom build cookbook generator to the cache/)
  when "git_repo"
    step %(the output should match /Skipping: Using cached copy of custom build cookbook generator/)
  else
    pending "not implemented"
  end
  step %(the output should contain "Custom build cookbook generated at .delivery/build_cookbook.")
  step %(the output should match /Feature branch named 'add-delivery-config' created./)
  step %(the output should match /Custom build cookbook committed to feature branch./)
  step %("git push --porcelain --progress --verbose delivery add-delivery-config:_for/master/add-delivery-config" should be run)
  step %(the file ".delivery/build_cookbook/test_file" should contain "THIS IS ONLY A TEST.")
  step %("git commit -m Adds Delivery build cookbook" should be run)
  step %(a directory named ".delivery/build_cookbook" should exist)
end

Given(/^both a custom build_cookbook and custom config is generated$/) do
  step %(the output should match /Custom Delivery config copied from .* to .*/)
  step %("git commit -m Adds Delivery build cookbook and config" should be run)
  step %(the output should match /Copying custom build cookbook generator to the cache/)
  step %(the output should contain "Custom build cookbook generated at .delivery/build_cookbook.")
  step %(the output should match /Feature branch named 'add-delivery-config' created./)
  step %(the output should match /Custom build cookbook committed to feature branch./)
  step %("git push --porcelain --progress --verbose delivery add-delivery-config:_for/master/add-delivery-config" should be run)
  step %(the file ".delivery/build_cookbook/test_file" should contain "THIS IS ONLY A TEST.")
  step %(a directory named ".delivery/build_cookbook" should exist)
end

Then(/^no build_cookbook is generated$/) do
  step %("git commit -m Adds Delivery build cookbook" should not be run)
  step %("chef generate cookbook .delivery/build_cookbook" should not be run)
  step %(a directory named ".delivery/build_cookbook" should not exist)
end

Then("I should be checked out to a feature branch named \"$name\"") do |name|
  step %q(I successfully run `git branch --contains HEAD`)
  step %Q(the output should contain "#{name}")
end

Then("a change should be created for branch \"$branch\"") do |branch|
  expected_command = "git push --porcelain --progress --verbose delivery #{branch}:_for/master/#{branch}"
  assert_command_run(expected_command)
end

Given(/^a dummy Delivery API server/) do
  endpoints = %(
    get('/api/v0/e/dummy/scm-providers') do
      status 200
      [
        {
          "name" => "GitHub",
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
    get('/api/v0/e/dummy/orgs') do
      status 200
      { "orgs" => ["dummy"] }
    end
    get('/api/v0/e/dummy/orgs/dummy/projects/already-created') do
      status 200
    end
    post('/api/v0/e/dummy/orgs/dummy/projects/already-created/pipelines') do
      status 201
      { "pipeline" => "master" }
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

    desc 'Create an organization.'
    params do
      requires :name, type: String, desc: 'Org name'
    end
    post '/api/v0/e/dummy/orgs' do
      if params[:name] == "new_org"
              status 201
      elsif params[:name] == "existing_org"
              status 409
      else
              status 500
      end
    end

    desc 'Request a token for link user.'
    post('/api/v0/e/dummy/users/link/get-token') do
      status 200
      {
         "token" => "xOsqI8qiBrUCGGRttfFy768R8ZAMJ24RC+0UGyX9/II=",
         "_links" => {
           "revoke-user-token" => {
             "href" => "/v0/e/bar/users/link/revoke-token"
           }
         }
      }
    end

    desc 'Delete ganondorf organization'
    delete('/api/v0/e/dummy/orgs/ganondorf') do
      status 204
    end
    desc 'Invalid endpoint'
    get('/api/v0/e/dummy/not_found') do
      status 404
    end
    desc 'Endpoint with unknown status code'
    get('/api/v0/e/dummy/endpoint_with_unknown_status_code') do
      status 429
    end
  )
  step %(the Delivery API server on port "8080":), endpoints
end

Given(/^a dummy A2 Workflow API server/) do
  endpoints = %(
    get('/workflow/api/v0/e/dummy/scm-providers') do
      status 200
      [
        {
          "name" => "GitHub",
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
                  "href" => "https://127.0.0.1//workflow/api/v0/e/skinkworks/bitbucket-servers/dummy.bitbucket.com"
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
    get('/workflow/api/v0/e/dummy/orgs') do # USED
      status 200
      { "orgs" => ["dummy"] }
    end
    get('/workflow/api/v0/e/dummy/orgs/dummy/projects/already-created') do
      status 200
    end
    post('/workflow/api/v0/e/dummy/orgs/dummy/projects/already-created/pipelines') do
      status 201
      { "pipeline" => "master" }
    end
    get('/workflow/api/v0/e/dummy/orgs/dummy/projects/delivery-cli-init') do
      status 201
      { "error" => "not_found" }
    end
    post('/workflow/api/v0/e/dummy/orgs/dummy/projects') do
      status 201
      { "project" => "delivery-cli-init" }
    end
    post('/workflow/api/v0/e/dummy/orgs/dummy/projects/delivery-cli-init/pipelines') do
      status 201
      { "pipeline" => "master" }
    end
    post('/workflow/api/v0/e/dummy/orgs/dummy/bitbucket-projects') do
      status 201
      { "project" => "delivery-cli-init" }
    end
    post('/workflow/api/v0/e/dummy/orgs/dummy/github-projects') do
      status 201
      { "project" => "delivery-cli-init" }
    end

    desc 'Create an organization.'
    params do
      requires :name, type: String, desc: 'Org name'
    end
    post '/workflow/api/v0/e/dummy/orgs' do
      if params[:name] == "new_org"
              status 201
      elsif params[:name] == "existing_org"
              status 409
      else
              status 500
      end
    end

    desc 'Request a token for link user.'
    post('/workflow/api/v0/e/dummy/users/link/get-token') do
      status 200
      {
         "token" => "xOsqI8qiBrUCGGRttfFy768R8ZAMJ24RC+0UGyX9/II=",
         "_links" => {
           "revoke-user-token" => {
             "href" => "/v0/e/bar/users/link/revoke-token"
           }
         }
      }
    end

    desc 'Delete ganondorf organization'
    delete('/workflow/api/v0/e/dummy/orgs/ganondorf') do
      status 204
    end
    desc 'Invalid endpoint'
    get('/workflow/api/v0/e/dummy/not_found') do
      status 404
    end
    desc 'Endpoint with unknown status code'
    get('/workflow/api/v0/e/dummy/endpoint_with_unknown_status_code') do
      status 429
    end
  )
  step %(the Delivery API server on port "8080":), endpoints
end
