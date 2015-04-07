module Chef_Delivery
  class ClientHelper
    class << self
      attr_reader :stored_config

      def leave_client_mode_as_delivery
        Chef::Log.info("Leaving client mode as delivery")
        Chef::Config.restore(@stored_config)
      end

      def enter_client_mode_as_delivery
        Chef::Log.info("Entering client mode as delivery")
        @stored_config = Chef::Config.save
        if File.exists?(File.expand_path(File.join('/var/opt/delivery/workspace/.chef', 'knife.rb')))
          Chef::Config.from_file(File.expand_path(File.join('/var/opt/delivery/workspace/.chef', 'knife.rb')))
        else
          Chef::Config.from_file(File.expand_path(File.join('/var/opt/delivery/workspace/etc', 'delivery.rb')))
        end
      end
    end
  end
end
