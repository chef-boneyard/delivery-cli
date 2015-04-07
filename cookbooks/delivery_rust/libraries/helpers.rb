module DeliveryRust
  module Helpers
    extend self

    # Using identifying components of the change, generate a project slug.
    #
    # @param [Chef::Node] Chef Node object
    # @return [String]
    def project_slug(node)
      change = node['delivery']['change']
      ent = change['enterprise']
      org = change['organization']
      proj = change['project']
      "#{ent}-#{org}-#{proj}"
    end

    # Pull down the encrypted data bag containing the secrets for this project.
    #
    # @param [Chef::Node] Chef Node object
    # @return [Hash]
    def get_project_secrets(node)
      ::Chef_Delivery::ClientHelper.enter_client_mode_as_delivery
      secret_file = Chef::EncryptedDataBagItem.load_secret(Chef::Config[:encrypted_data_bag_secret])
      secrets = Chef::EncryptedDataBagItem.load('delivery-secrets', project_slug(node), secret_file)
      ::Chef_Delivery::ClientHelper.leave_client_mode_as_delivery
      secrets
    end
  end

  module DSL
    def get_project_secrets
      DeliveryRust::Helpers.get_project_secrets(node)
    end
  end
end
