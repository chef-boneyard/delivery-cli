# cbs_to_release = ["delivery_builder", "simple_build", "delivery-server"]
# git_ssh = File.join(node['delivery_builder']['root_workspace_bin'], 'git_ssh')
# 
# cookbook_directory = File.join(node['delivery_builder']['cache'], "cookbook-upload")
# directory cookbook_directory
# 
# env_name = "acceptance-#{node['delivery_builder']['change']['project']}-#{node['delivery_builder']['change']['pipeline']}"
# 
# ## Update Github
# execute "Set Git Username" do
#   command "git config --global user.name 'Delivery'"
#   cwd node['delivery_builder']['repo']
#   environment({"GIT_SSH" => git_ssh})
#   user node['delivery_builder']['build_user']
# end
# 
# execute "Set Git Email" do
#   command "git config --global user.email 'delivery@getchef.com'"
#   cwd node['delivery_builder']['repo']
#   environment({"GIT_SSH" => git_ssh})
#   user node['delivery_builder']['build_user']
# end
# 
# execute "Add Github Remote" do
#   command "git remote add github git@github.com:opscode/delivery.git"
#   cwd node['delivery_builder']['repo']
#   environment({"GIT_SSH" => git_ssh})
#   user node['delivery_builder']['build_user']
#   returns [0,128]
# end
# 
# execute "Push To Github" do
#   command "git push github master"
#   cwd node['delivery_builder']['repo']
#   environment({"GIT_SSH" => git_ssh})
#   user node['delivery_builder']['build_user']
# end
# 
# ruby_block "Create Env #{env_name} if not there." do
#   block do
#     Chef_Delivery::ClientHelper.enter_client_mode_as_delivery
# 
#     begin
#       env = Chef::Environment.load(env_name)
#     rescue Net::HTTPServerException => http_e
#       raise http_e unless http_e.response.code == "404"
#       Chef::Log.info("Creating Environment #{env_name}")
#       env = Chef::Environment.new()
#       env.name(env_name)
#       env.create
#     end
#     Chef_Delivery::ClientHelper.enter_solo_mode
#   end
# end
# 
# cbs_to_release.each do |cb|
#   cb_dir = ::File.join(node['delivery_builder']['repo'], "/infra/cookbooks/", cb)
#   metadata = Chef::Cookbook::Metadata.new
#   metadata.from_file(::File.expand_path(::File.join(cb_dir, "metadata.rb")))
# 
#   Chef_Delivery::ClientHelper.enter_client_mode_as_delivery
#   env = Chef::Environment.load(env_name)
#   Chef_Delivery::ClientHelper.enter_solo_mode
# 
#   if env.cookbook_versions[cb] != "= #{metadata.version}"
#     link File.join(cookbook_directory, cb) do
#       to cb_dir
#     end
# 
#     delivery_builder_exec "knife cookbook upload #{cb} --environment #{env_name} --freeze -c #{File.join(node['delivery_builder']['root_workspace_etc'], 'delivery.rb')} -o #{cookbook_directory}"
#   end
# end
