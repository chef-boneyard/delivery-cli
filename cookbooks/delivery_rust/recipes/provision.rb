
#########################################################################
# Ideally the following code would execte in `build/publish` after all
# builders in the matrix have completed publishing. Since Delivery doesn't
# support these types of hooks yet we'll create our build record first
# thing in the acceptance stage.
#########################################################################

#
# Ensure we are executing in acceptance/provision
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
  # Create an Artifactory build record that groups all artifacts
  # published to `omnibus-unstable-local` in `build/publish`. This
  # record is used to execute promotions further down the pipeline.
  #######################################################################
  artifactory_build_record "delivery-cli##{delivery_change_id}" do
    properties(
      'omnibus.project' => 'delivery-cli',
      'delivery.change' => delivery_change_id,
    )
  end
end
