# Build `delivery-cli` on Windows
This document describes the process to build a `delivery-cli` package on windows systems.
We will be using Test-Kitchen so you can build it on any system because we will be spinning
up the windows machines that will build the package.

_NOTE: This document should be removed when our Delivery pipeline builds this packages automatically._

Here are the steps:

#### Clone `delivery-cli` repository
Clone the repository locally on your computer
```
$ git clone https://github.com/chef/delivery-cli.git
```

#### Install and prepare omnibus (locally)
```
$ cd delivery-cli/omnibus-delivery-cli
$ bundle install
```

#### Converge a Windows Instance (via kitchen)
You can use any windows machine that is shown by the `kitchen list` command.
```
$ bundle exec kitchen converge windows-2012r2
```

#### Copy repository locally on the windows machine
Omnibus doesn't work fine when we use a network device inside the virtual machine,
therefor you have to copy the repository locally inside the windows machine.
You will have the path `C:\home\vagrant\delivery-cli`, make a copy of it and put it
at the same parent directory.

#### Load omnibus variables and build the package
Finally you have to open a CMD terminal inside the windows machine and load the
environment variables that omnibus make available for you at:
```
C:\> C:\home\vagrant\cache\load-omnibus-toolchain.bat
```

Prepare and install gem dependencies:
```
C:\> cd C:\home\vagrant\delivery-cli\omnibus-delivery-cli
C:\home\vagrant\delivery-cli\omnibus-delivery-cli> bundle install --binstubs
```

Build `delivery-cli` package: (MSI)
```
C:\home\vagrant\delivery-cli\omnibus-delivery-cli> bundle exec omnibus build delivery-cli --log-level internal
```

You will find the package at: `C:\home\vagrant\delivery-cli\omnibus-delivery-cli\pkg`

