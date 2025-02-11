# mcvcli - Minecraft Server Version Manager

mcvcli is a command-line tool for managing Minecraft server versions. It allows you to easily download, install, and switch between different versions of the Minecraft server software.

## Features

- Download and install Minecraft server versions with a single command
- List available server versions
- Switch between installed server versions
- Automatically handle java installation

## Installation

### Using Cargo

1. Make sure you have [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed.
2. Install mcvcli globally by running the following command:

```bash
cargo install mcvcli
```

### Using a Pre-built Binary

1. Download the latest release from the [releases page](https://github.com/mcjars/mcvcli/releases).
2. Extract the downloaded archive.
3. Add the extracted directory to your `PATH` (or put it in a folder that is already in `PATH`).
4. Run `mcvcli` in your terminal to verify that the installation was successful.

```bash
# Basic Example for Linux
wget https://github.com/mcjars/mcvcli/releases/latest/download/mcvcli-x86_x64-linux.tar.xz
tar -xzf mcvcli-x86_x64-linux.tar.xz -C .
export PATH=$PATH:$(pwd)/mcvcli-x86_x64-linux

mcvcli --version
```

```powershell
# Basic Example for Windows

Invoke-WebRequest -Uri "https://github.com/mcjars/mcvcli/releases/latest/download/mcvcli-x86_x64-windows.zip" -OutFile "mcvcli-x86_x64-windows.zip"
Expand-Archive -Path "mcvcli-x86_x64-windows.zip" -DestinationPath "."
$env:Path += ";$(Get-Location)\mcvcli-x86_x64-windows"

mcvcli --version
```

## Usage

### Downloading and Installing a Server Version

To setup your Minecraft server version, use the `init` command

```bash
mcvcli init ./server

cd server

mcvcli version # view installed version, auto updates with your jar
mcvcli update # update build or minecraft version of your jar (only newer)
mcvcli install # force install any other version
mcvcli lookup {user} # lookup a user on your server or globally
mcvcli start # start the server
mcvcli profile list # list server profiles
mcvcli profile create {name} # create a new profile
mcvcli profile use {name} # switch to another profile
mcvcli profile delete {name} # nuke a profile from existance
mcvcli backup list # list created server backups
mcvcli backup create # create a new server backup
mcvcli backup delete # delete a server backup
mcvcli backup restore # restore a previously created server backup
```

## Developing

To Develop on this tool, you need to install all required dependencies

```bash
git clone https://github.com/mcjars/mcvcli.git mcjars-mcvcli

cd mcjars-mcvcli

# make sure to have cargo installed already
cargo build

# install binary globally
cargo install --path .

# run the binary temporarily
cargo run -- --version
```

> [!NOTE]
> NOT AN OFFICIAL MINECRAFT SERVICE. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT.
