# Mock a custom config.json
def custom_config
<<EOF
{
  "version": "2",
  "build_cookbook": {
    "path": ".delivery/build_cookbook",
    "name": "build_cookbook"
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

# Mock a build_cookbook.rb that doesn't generate a config.json
def build_cookbook_rb
<<EOF
context = ChefDK::Generator.context
delivery_project_dir = context.delivery_project_dir
dot_delivery_dir = File.join(delivery_project_dir, ".delivery")
directory dot_delivery_dir
build_cookbook_dir = File.join(dot_delivery_dir, "build_cookbook")
directory build_cookbook_dir
template "\#{build_cookbook_dir}/metadata.rb" do
  source "build_cookbook/metadata.rb.erb"
  helpers(ChefDK::Generator::TemplateHelper)
  action :create_if_missing
end
template "\#{build_cookbook_dir}/Berksfile" do
  source "build_cookbook/Berksfile.erb"
  helpers(ChefDK::Generator::TemplateHelper)
  action :create_if_missing
end
directory "\#{build_cookbook_dir}/recipes"
%w(default deploy functional lint provision publish quality security smoke syntax unit).each do |phase|
  template "\#{build_cookbook_dir}/recipes/\#{phase}.rb" do
    source 'build_cookbook/recipe.rb.erb'
    helpers(ChefDK::Generator::TemplateHelper)
    variables phase: phase
    action :create_if_missing
  end
end
EOF
end

# Mock a cli.toml config
def basic_delivery_config
<<EOF
git_port = "8080"
pipeline = "master"
user = "dummy"
server = "127.0.0.1:8080"
enterprise = "dummy"
organization = "dummy"
EOF
end

# Mock default delivery config.json
def default_delivery_config
<<EOF
  {
    "version": "2",
    "build_cookbook": {
      "path": ".delivery/build_cookbook",
      "name": "build_cookbook"
    },
    "skip_phases": [],
    "build_nodes": {},
    "dependencies": []
  }
EOF
end

# Mock basic git config
def basic_git_config
<<EOF
[config]
EOF
end

def additional_gen_recipe
<<EOF
file "\#{build_cookbook_dir}/test_file" do
  content 'THIS IS ONLY A TEST.'
end
EOF
end

# Relative path of a temporal directory
def tmp_relative_path
  @tmp_relative_dir ||= '../tmp'
  step %(a directory named "#{@tmp_relative_dir}")
  @tmp_relative_dir
end

# Absolute  path of a temporal directory
def tmp_expanded_path
  @tmp_expanded_dir ||= expand_path('../tmp')
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

When(/^I wait for (\d+) seconds?$/) do |n|
    sleep(n.to_i)
end

Given(/^a user creates a delivery backed project$/) do
  step %(I successfully run `delivery init`)
end

Given(/^a user creates a github backed project$/) do
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

Given(/^a delivery project is created in delivery$/) do
  step %(the output should match /Delivery project named.*was created/)
  step %(the output should contain "Remote 'delivery' added as")
end

Given(/^a delivery project should not be created in delivery$/) do
  step %(the output should not match /Delivery project named.*was created/)
  step %(the output should match /Delivery project named .* already exists/)
  step %(the output should contain "Remote 'delivery' added as")
end

Given(/^a bitbucket project is created in delivery$/) do
  step %(the output should match /Bitbucket backed Delivery project named .* created./)
  step %(the output should contain "Remote 'delivery' added as")
end

Given(/^a github project is created in delivery$/) do
  step %(the output should match /Creating.*github.*project/)
  step %(the output should not contain "Remote 'delivery' added as")
end

Given(/^a default config.json is created$/) do
  step %(the file ".delivery/config.json" should contain:), %("version": "2",)
  step %(the file ".delivery/config.json" should contain:), %("build_cookbook": {)
  step %(the file ".delivery/config.json" should contain:), %("path": ".delivery/build_cookbook")
  step %(the file ".delivery/config.json" should contain:), %("name": "build_cookbook")
  step %(the file ".delivery/config.json" should contain:), %(},)
  step %(the file ".delivery/config.json" should contain:), %("skip_phases": [],)
  step %(the file ".delivery/config.json" should contain:), %("build_nodes": {},)
  step %(the file ".delivery/config.json" should contain:), %("dependencies": [])
end

Given(/^a change to the delivery config is not comitted$/) do
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

Given(/^a change configuring a custom delivery is created$/) do
  step %("git checkout -b add-delivery-config" should be run)
  step %("git commit -m Adds custom Delivery config" should be run)
  step %(the file ".delivery/config.json" should contain exactly:), custom_config
end

Given(/^the change has the default generated build_cookbook$/) do
  step %("git push --set-upstream --porcelain --progress --verbose delivery master" should be run)
  step %("git commit -m Add generated delivery configuration" should be run)
  step %("git commit -m Add generated delivery build cookbook" should be run)
  step %(a directory named ".delivery/build_cookbook" should exist)
end

Given(/^the change does not have the default generated build_cookbook$/) do
  step %("git commit -m Adds Delivery build cookbook and config" should not be run)
  step %("chef generate cookbook .delivery/build_cookbook" should not be run)
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
  )
  step %(the Delivery API server on port "8080":), endpoints
end
