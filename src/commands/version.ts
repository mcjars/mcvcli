import fs from "fs"
import path from "path"
import chalk from "chalk"
import * as api from "src/api"
import getConfig from "src/utils/config"
import getJarVersion from "src/utils/jar"

export type Args = {
	profile?: string
}

export default async function version(args: Args) {
	if (args.profile) {
		if (!fs.existsSync(path.join('.mccli.profiles', args.profile))) {
			console.log('profile not found!')
			process.exit(1)
		}

		process.chdir(path.join('.mccli.profiles', args.profile))

		console.log('checking installed version in profile...')
	} else {
		console.log('checking currently installed version...')
	}

	const config = getConfig(),
		version = await getJarVersion(path.resolve(config.data.jarFile))

	const { latestJar, latestMc } = await api.latest(version.type, version.minecraftVersion!)

	console.log('installed jar location:', chalk.cyan(config.data.jarFile))
	console.log('installed jar version:')
	console.log('  type:', chalk.cyan(version.type))
	if (version.minecraftVersion) console.log('  minecraft version:', chalk.cyan(version.minecraftVersion), latestMc === version.minecraftVersion ? chalk.green('(latest)') : chalk.red('(outdated)'))
	if (version.jarVersion) console.log('  jar version:', chalk.cyan(version.jarVersion), latestJar === version.jarVersion ? chalk.green('(latest)') : chalk.red('(outdated)'))

	if (args.profile) process.chdir('../..')
}