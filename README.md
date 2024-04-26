# mcvcli - Minecraft Server Version Manager

mcvcli is a command-line tool for managing Minecraft server versions. It allows you to easily download, install, and switch between different versions of the Minecraft server software.

## Features

- Download and install Minecraft server versions with a single command
- List available server versions
- Switch between installed server versions
- Automatically handle server configuration files
- Automatically handles java installation

## Installation

1. Make sure you have [Node.js](https://nodejs.org) installed on your system.
2. Install mcvcli globally by running the following command:

```bash
npm install -g mcvcli
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
```

## Developing

To Develop on this tool, you need to install all required dependencies

```bash
git clone https://github.com/0x7d8/mcvcli

cd mcvcli

# make sure to have nodejs installed already
npm i -g pnpm
pnpm i
pnpm install:dev

# mcvcli is now globally available
mcvcli
```
