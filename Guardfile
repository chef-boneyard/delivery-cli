# A sample Guardfile
# More info at https://github.com/guard/guard#readme

# Add files and commands to this file, like the example:
#   watch(%r{file/path}) { `command(s)` }
#
guard :shell do
  watch(/(.*).rs/) do |m|
    if system("cargo test")
      n "#{m[0]} is ok", 'Cargo Test', :success
    else
      n "#{m[0]} is not ok", 'Cargo Test', :failed
    end
  end
end
