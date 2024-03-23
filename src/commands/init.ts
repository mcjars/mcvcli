import fs from "fs"
import path from "path"
import enquirer from "enquirer"
import chalk from "chalk"
import * as api from "src/api"
import getJarVersion from "src/utils/jar"
import { Config } from "src/utils/config"

export type Args = {
	directory: string
}

export default async function init(args: Args, profileName?: string) {
  if (!fs.existsSync(args.directory)) {
    fs.mkdirSync(args.directory)
  }

  const previousCwd = process.cwd()
  process.chdir(args.directory)

  if (fs.existsSync('.mccli.json')) {
    console.log('mccli is already initialized!')
    console.log('use', chalk.cyan('mccli version'), 'to get the currently installed version')

    process.chdir(previousCwd)
    process.exit(1)
  }

  const files = fs.readdirSync('.')
  
  const { jarFile } = await enquirer.prompt<{
    jarFile: string
  }>({
    type: 'select',
    message: 'Server Jar File',
    name: 'jarFile',
    choices: [
      'Install New',
      ...files.filter((file) => file.endsWith('.jar'))
    ]
  })

  switch (jarFile) {
    case "Install New": {
      const { type } = await enquirer.prompt<{
        type: api.SupportedProject
      }>({
        type: 'select',
        message: 'Server Type',
        name: 'type',
        choices: [...api.supportedProjects]
      })

      console.log('checking versions...')

      const versions = await api.versions(type),
        { version } = await enquirer.prompt<{
          version: string
        }>({
          type: 'autocomplete',
          message: 'Server Version',
          name: 'version',
          choices: versions.reverse(),
          // @ts-ignore
          limit: 10
        })

      console.log('checking latest build...')

      const builds = await api.builds(type, version),
        latest = builds[0]

      const { ramMB } = await enquirer.prompt<{
        ramMB: number
      }>({
        type: 'numeral',
        message: 'Server RAM (MB)',
        name: 'ramMB',
        min: 1024,
        initial: 4096
      })

      const config = new Config({
        __README: 'This file is used to store the configuration for the mccli tool. Do not modify this file unless you know what you are doing.',
        jarFile: 'server.jar',
        profileName: profileName ?? 'default',
        ramMB
      })

      await api.install(latest.download, config)
      config.write()

      break
    }

    default: {
      console.log('checking installed version...')

      const version = await getJarVersion(path.resolve(jarFile))

      const { latestJar, latestMc } = await api.latest(version.type, version.minecraftVersion!)

      console.log('installed jar location:', chalk.cyan(jarFile))
      console.log('installed jar version:')
      console.log('  type:', chalk.cyan(version.type))
      if (version.minecraftVersion) console.log('  minecraft version:', chalk.cyan(version.minecraftVersion), latestMc === version.minecraftVersion ? chalk.green('(latest)') : chalk.red('(outdated)'))
      if (version.jarVersion) console.log('  jar version:', chalk.cyan(version.jarVersion), latestJar === version.jarVersion ? chalk.green('(latest)') : chalk.red('(outdated)'))

      const { ramMB } = await enquirer.prompt<{
        ramMB: number
      }>({
        type: 'numeral',
        message: 'Server RAM (MB)',
        name: 'ramMB',
        min: 1024,
        initial: 4096
      })

      const config = new Config({
        __README: 'This file is used to store the configuration for the mccli tool. Do not modify this file unless you know what you are doing.',
        jarFile,
        profileName: profileName ?? 'default',
        ramMB
      })

      config.write()

      break
    }
  }

	if (!profileName) fs.mkdirSync(path.join('.mccli.profiles', 'default'), { recursive: true })
  process.chdir(previousCwd)
  console.log('mccli initialized!')
}