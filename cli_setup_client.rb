# Only used by the `make setup` target to ensure we pull from the
# appropriate directory, independent of any other Chef configuration
# files that may be on the system
current_dir = File.absolute_path(File.dirname(__FILE__))
chef_repo_path current_dir
cookbook_path ["#{current_dir}/vendor/cookbooks"]
file_cache_path File.join(current_dir, 'local-mode-cache', 'cache')
