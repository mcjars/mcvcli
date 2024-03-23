# mccli - Minecraft Server Version Manager

mccli is a command-line tool for managing Minecraft server versions. It allows you to easily download, install, and switch between different versions of the Minecraft server software.

## Features

- Download and install Minecraft server versions with a single command
- List available server versions
- Switch between installed server versions
- Automatically handle server configuration files

## Installation

1. Make sure you have [Node.js](https://nodejs.org) installed on your system.
2. Install mccli globally by running the following command:

```bash
npm install -g mccli
```

## Usage

### Downloading and Installing a Server Version

To setup your Minecraft server version, use the `init` command

```bash
mccli init ./server

cd server

mccli version # view installed version, auto updates with your jar
mccli update # update build or minecraft version of your jar (only newer)
mccli install # force install any other version
mccli lookup {user} # lookup a user on your server or globally
mccli start # start the server
mccli profile list # list server profiles
mccli profile create {name} # create a new profile
mccli profile use {name} # switch to another profile
mccli profile delete {name} # nuke a profile from existance
```
