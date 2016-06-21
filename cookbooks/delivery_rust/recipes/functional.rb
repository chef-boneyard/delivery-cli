# TODO: We need some actual acceptance tests here!

if node['delivery']['change']['stage'] == 'delivered'
  #########################################################################
  # PUBLISH TAGS
  #########################################################################

  git_ssh = File.join('/var/opt/delivery/workspace/bin', 'git_ssh')
  execute "Add Github Remote" do
    command "git remote add github git@github.com:opscode/delivery-cli.git"
    cwd node['delivery']['workspace']['repo']
    environment({"GIT_SSH" => git_ssh})
    returns [0,128]
  end

  execute "Fetch Tags" do
    command "git fetch --tags"
    cwd node['delivery']['workspace']['repo']
    environment({"GIT_SSH" => git_ssh})
    returns [0]
  end

  ## Doing this in a ruby block so I can get the current tag
  cmd = "git tag -l | sort -V | tail -n1 |awk -F \".\" '{ print $1 \".\" $2 \".\" $3 + 1 }'"
  Chef::Log.error("#{cmd}")
  ruby_block "Tag Release" do
    block do
      Dir.chdir(node['delivery']['workspace']['repo']) do
        ENV['GIT_SSH'] = git_ssh
        tag = `#{cmd}`.strip
        if tag == ''
          tag = '0.0.1'
        end
        `git tag #{tag} -a -m "Delivery Cli #{tag}"`
      end
    end
  end

  execute "Push Tags Delivery" do
    command "git push origin --tags"
    cwd node['delivery']['workspace']['repo']
    environment({"GIT_SSH" => git_ssh})
  end

  execute "Push Tags Github" do
    command "git push github --tags"
    cwd node['delivery']['workspace']['repo']
    environment({"GIT_SSH" => git_ssh})
  end

  #########################################################################
  # PUBLISH TO GITHUB
  #########################################################################

  delivery_bus_secrets = DeliverySugar::ChefServer.new.encrypted_data_bag_item('delivery-bus', 'secrets')

  delivery_github 'Push delivery-cli to GitHub' do
    repo_path delivery_workspace_repo
    cache_path delivery_workspace_cache
    deploy_key delivery_bus_secrets['github_private_key'] # chef-delivery's key
    remote_name 'github'
    remote_url 'git@github.com:chef/delivery-cli.git'
  end
end
