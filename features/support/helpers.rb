#
# Helper methods
#

# Mock a custom config.json
def custom_config
<<EOF
{
  "version": "2",
  "build_cookbook": {
    "path": ".delivery/build_cookbook",
    "name": "build_cookbook"
  },
  "skip_phases": [ "smoke", "security", "syntax", "unit", "quality" ],
  "job_dispatch": {
    "version": "v2"
  },
  "delivery-truck": {
    "publish": {
      "chef_server": true
    }
  },
  "dependencies": []
}
EOF
end

def config_with_custom_build_cookbook
<<EOF
{
  "version": "2",
  "build_cookbook": {
    "path": "cookbooks/bubulubu",
    "name": "bubulubu"
  },
  "skip_phases": [],
  "job_dispatch": {
    "version": "v2"
  },
  "dependencies": []
}
EOF
end

# Mock a config.json where the source of the build_cookbook is Supermarket
def config_build_cookbook_from_supermarket
<<EOF
{
  "version": "2",
  "build_cookbook": {
    "supermarket": "true",
    "name": "vikings"
  },
  "skip_phases": [],
  "job_dispatch": {
    "version": "v2"
  },
  "dependencies": []
}
EOF
end

# Mock a project.toml that has missing phases
def incomplete_project_toml
<<EOF
# Missing an 's' in phases
[local_phase]
lint = "echo 'This file is wrong, we have missing phases'"
EOF
end

# Mock a project.toml that has failures on phases
def project_toml_with_failures
<<EOF
[local_phases]
lint = "foodcritic failure"
syntax = "chefstyle failure"
unit = "rspec failure"
EOF
end

# Mock a project.toml that is partially config
def partial_project_toml
<<EOF
[local_phases]
lint = "echo 'This file is valid'"
EOF
end

# Mock a project.toml
def project_toml
<<EOF
[local_phases]
unit = "echo 'This is a cool unit test'"
lint = "cookstyle"
syntax = "foodcritic . -t ~supermarket"
provision = "echo 'Creating instances'"
deploy = "echo 'Converging instances'"
smoke = "echo 'Smoking tests'"
functional = "echo 'Functional tests'"
cleanup = "echo 'Cleaning up'"
EOF
end

def dummy_cli_toml
<<EOF
api_protocol = "https"
enterprise = "dummy"
api_port = "8080"
organization = "zelda"
server = "localhost"
user = "link"
EOF
end

def dummy_a2_cli_toml
<<EOF
api_protocol = "https"
enterprise = "dummy"
api_port = "8080"
organization = "zelda"
server = "localhost"
user = "link"
a2_mode = true
EOF
end

def valid_cli_toml
<<EOF
api_protocol = "https"
enterprise = "ent"
git_port = "8989"
organization = "org"
pipeline = "master"
server = "server.test"
user = "user"
EOF
end

def remote_project_toml
<<EOF
[local_phases]
unit = "echo REMOTE-UNIT"
lint = "echo REMOTE-LINT"
syntax = "echo REMOTE-SYNTAX"
provision = "echo REMOTE-PROVISION"
deploy = "echo REMOTE-DEPLOY"
smoke = "echo REMOTE-SMOKE"
cleanup = "echo REMOTE-CLEANUP"
EOF
end

def project_toml_with_remote_file(url)
<<EOF
remote_file = "#{url}"
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
# Starts a server on 8080 so the git port is also 8080,
# so don't be surprised to see addresses like 127.0.0.1:8080:8080
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
    "job_dispatch": {
      "version": "v2"
    },
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

# Absolute path of a temporal directory
def tmp_expanded_path
  @tmp_expanded_dir ||= expand_path('../tmp')
end
