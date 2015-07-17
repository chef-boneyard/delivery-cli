# We want it to be possible to build in a stand-alone fashion, not
# just on official Delivery build nodes
dbuild_user_exists = system("grep -q dbuild /etc/passwd")

include_recipe "omnibus::_packaging"
include_recipe "omnibus::_selinux"

directory "/opt/delivery-cli" do
  owner 'dbuild'
  only_if { dbuild_user_exists }
end

# Make sure all the files are owned by us - keeps us safe after
# package upgrades
execute "chown -R dbuild /opt/delivery-cli" do
  only_if { dbuild_user_exists }
  only_if "test -d /opt/delivery-cli"
end

# Make a backup so that if the build fails, we can rescue ourselves
execute "rsync -aP --delete /opt/delivery-cli/ /opt/delivery-cli-safe" do
  only_if "test -f /opt/delivery-cli/bin/delivery"
end
