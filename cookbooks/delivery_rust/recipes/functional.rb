
# TODO: We need some actual acceptance tests here!

#
# Ensure we are executing in acceptance/functional
#
# TODO: add a `delivery_stage?(stage)` helpers to delivery-sugar's DSL
if node['delivery']['change']['stage'] == 'acceptance'
  #######################################################################
  # TODO: set these things in `delivery-bus`
  #######################################################################
  delivery_bus_secrets = DeliverySugar::ChefServer.new.encrypted_data_bag_item('delivery-bus', 'secrets')

  node.set['artifactory-pro']['endpoint']      = 'http://artifactory.chef.co:8081'
  node.run_state[:artifactory_client_username] = delivery_bus_secrets['artifactory_username']
  node.run_state[:artifactory_client_password] = delivery_bus_secrets['artifactory_password']

  #######################################################################
  # Once tests have passed acceptance testing passed we can safely
  # promote the build from the `unstable` to the `current` channel.
  #######################################################################
  chef_artifactory_promotion "promote delivery-cli##{delivery_change_id} to current" do
    omnibus_project 'delivery-cli'
    delivery_change delivery_change_id
    channel :current
    comment "Promoted by Delivery change #{delivery_change_id} during acceptance/functional"
    user 'dbuild'
  end
end
