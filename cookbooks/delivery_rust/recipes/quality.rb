#########################################################################
# PUBLISH TAGS
#########################################################################
# Create a tag in this phase group so that we can
# use that tag for versioning in the publish recipe.
# That version will then be consistent across all
# all platforms for all built artifacts.
# Phase groups run atomically. Quality and security
# will complete before before publish runs, yielding
# a consistent version.
git_ssh = File.join('/var/opt/delivery/workspace/bin', 'git_ssh')
    execute "Add Github Remote" do
      command "git remote add github git@github.com:opscode/delivery.git"
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

