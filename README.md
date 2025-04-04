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
wget https://github.com/mcjars/mcvcli/releases/latest/download/mcvcli-x86_64-linux.tar.xz
tar xfv mcvcli-x86_64-linux.tar.xz -C .
mv mcvcli-x86_64-linux/mcvcli /usr/bin
chmod +x /usr/bin/mcvcli
rm -r mcvcli-x86_64-linux mcvcli-x86_64-linux.tar.xz

mcvcli --version
```

```powershell
# Basic Example for Windows

Invoke-WebRequest -Uri "https://github.com/mcjars/mcvcli/releases/latest/download/mcvcli-x86_64-windows.zip" -OutFile "mcvcli-x86_64-windows.zip"
Expand-Archive -Path "mcvcli-x86_64-windows.zip" -DestinationPath "." -Force
$env:Path += ";$(Get-Location)\mcvcli-x86_64-windows"

mcvcli --version
```

## Usage

### Downloading and Installing a Server Version

To setup your Minecraft server version, use the `init` command

```bash
mkdir /home/Minecraft
cd /home/Minecraft
mcvcli init .

mcvcli version # View installed version, auto updates with your jar
mcvcli update # Update build or minecraft version of your jar (only newer)
mcvcli install # Force install any other version
mcvcli lookup {user} # Lookup a user on your server or globally
mcvcli start # Start the server

mcvcli java list # List installed java versions
mcvcli java install {version} # Install a specific java version
mcvcli java use {version} # Switch to another java version
mcvcli java delete {version} # Remove a java version

mcvcli profile list # List server profiles
mcvcli profile create {name} # Create a new profile
mcvcli profile use {name} # Switch to another profile
mcvcli profile delete {name} # Nuke a profile from existance

mcvcli backup list # List created server backups
mcvcli backup create {name} # Create a new server backup
mcvcli backup delete {name} # Delete a server backup
mcvcli backup restore {name} # Restore a previously created server backup

mcvcli mods list # List installed mods
mcvcli mods delete # Delete selected mods

mcvcli start --detached # Start the server in the background (no output)
mcvcli attach # Attach to the server console
mcvcli stop # Stop the server
mcvcli status # Check the server status

mcvcli upgrade # Upgrade the mcvcli binary
```

## Developing

To Develop on this tool, you need to install all required dependencies

```bash
mkdir /home/mcvcli
cd /home/mcvcli
git init
git pull https://github.com/mcjars/mcvcli.git

# Install binary globally
cargo install --path .
mcvcli --version

# Run the binary temporarily
cargo run -- --version
```

> [!NOTE]
> NOT AN OFFICIAL MINECRAFT SERVICE. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT.