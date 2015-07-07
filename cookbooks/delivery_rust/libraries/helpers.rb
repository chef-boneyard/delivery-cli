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

    # Return the date of the installed nightly build of rustc or
    # "NONE" if rustc is not installed.
    def current_rust_version
      # output of `rustc --version` looks like this:
      # rustc 1.3.0-nightly (faa04a8b9 2015-06-30)
      %x(rustc --version).split.last[0..-2]
    rescue Errno::ENOENT
      "NONE"
    end
  end

  module DSL
    def get_project_secrets
      DeliveryRust::Helpers.get_project_secrets(node)
    end

    def current_rust_version
      DeliveryRust::Helpers.current_rust_version
    end
  end
end
