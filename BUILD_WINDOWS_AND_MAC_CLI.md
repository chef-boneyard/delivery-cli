# Build `delivery-cli` on Windows and Mac OS X
This document describes the process to artisanally build a `delivery-cli` package on platforms
that are not currently in the Delivery pipeline for this product. We will be using Test-Kitchen
so you can build it on any host system but these instructions have only been tested on a
Mac OS X host system. We also assume you are using the Test-Kitchen that ships in the latest
ChefDK release.

_NOTE: This document should be removed when our Delivery pipeline builds this packages automatically._

Here are the steps:

#### Clone `delivery-cli` repository
Clone the repository locally on your computer
```
$ git clone https://github.com/chef/delivery-cli.git
```

#### Converge a Windows Instance (via kitchen)
You can use any windows machine that is shown by the `kitchen list` command.

```
$ kitchen converge delivery-cli-windows-2012r2-standard
```

Finally you have to open a CMD.exe terminal inside the windows machine and load
the Omnibus build toolchain:

```
C:\> C:\home\vagrant\load-omnibus-toolchain.bat
```

Prepare and install gem dependencies:

```
C:\> cd C:\home\vagrant\code\delivery-cli\omnibus
C:\home\vagrant\code\delivery-cli\omnibus> bundle install
```

Build `delivery-cli` package: (MSI)

```
C:\home\vagrant\code\delivery-cli\omnibus> bundle exec omnibus build delivery-cli -l internal
```

You will find the `delivery-cli-*.msi` at: `C:\home\vagrant\code\delivery-cli\omnibus\pkg`

#### Converge a Mac OSX Instance (via kitchen)
You can use any Mac OS X machine that is shown by the `kitchen list` command.
You do have to use Vagrant's VMmware provider which can be activated with the
`.kitchen.vmware.yml` file that ships in this repo. You need to log onto the
desktop of the VM that is brought up (with the `vagrant` user) if you want DMG
creation to work correctly (as it uses AppleScript under the covers).

```
$ export KITCHEN_LOCAL_YAML=.kitchen.vmware.yml
$ kitchen converge delivery-cli-macosx-109
```

Next login to the instance and load the Omnibus build toolchain:

```
$ kitchen login delivery-cli-macosx-109
delivery-cli-macosx-109:~ vagrant$ source load-omnibus-toolchain.sh
```

Prepare and install gem dependencies:

```
delivery-cli-macosx-109:~ vagrant$ cd delivery-cli/omnibus
delivery-cli-macosx-109:omnibus vagrant$ bundle install
```

Build `delivery-cli` package: (DMG)

```
delivery-cli-macosx-109:omnibus vagrant$ bundle exec omnibus build delivery-cli -l internal
```

You will find the `delivery-cli-*.pkg` and `delivery-cli-*.dmg` at: `/Users/vagrant/delivery-cli/omnibus/pkg`
