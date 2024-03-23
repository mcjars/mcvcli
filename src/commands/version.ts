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

	const config = getConfig()

	if (!config.data.modpackSlug && !config.data.modpackVersion) {
		const version = await getJarVersion(path.resolve(config.data.jarFile))

		const { latestJar, latestMc } = await api.latest(version.type, version.minecraftVersion!)

		console.log('installed jar location:', chalk.cyan(config.data.jarFile))
		console.log('installed jar version:')
		console.log('  type:', chalk.cyan(version.type))
		if (version.minecraftVersion) console.log('  minecraft version:', chalk.cyan(version.minecraftVersion), latestMc === version.minecraftVersion ? chalk.green('(latest)') : chalk.red('(outdated)'))
		if (version.jarVersion) console.log('  jar version:', chalk.cyan(version.jarVersion), latestJar === version.jarVersion ? chalk.green('(latest)') : chalk.red('(outdated)'))
	} else if (config.data.modpackSlug && config.data.modpackVersion) {
		const { modpackSlug, modpackVersion } = config.data

		const [ { latestVersion }, versions, infos, jar ] = await Promise.all([
			api.latestModpack(modpackSlug),
			api.modpackVersions(modpackSlug),
			api.modpackInfos(modpackSlug),
			getJarVersion(path.resolve(config.data.jarFile))
		])

		const version = versions.find(v => v.id === modpackVersion)

		console.log('installed modpack:')
		console.log('  title:', chalk.cyan(infos.title))
		console.log('  license:', chalk.cyan(infos.license.id))
		console.log('  version:', chalk.cyan(version?.version_number ?? 'unknown'), latestVersion === version?.version_number ? chalk.green('(latest)') : chalk.red('(outdated)'))
		console.log('installed jar location:', chalk.cyan(config.data.jarFile))
		console.log('installed jar version:')
		console.log('  type:', chalk.cyan(jar.type))
		if (jar.minecraftVersion) console.log('  minecraft version:', chalk.cyan(jar.minecraftVersion), latestVersion === jar.minecraftVersion ? chalk.green('(latest)') : chalk.red('(outdated)'))
		if (jar.jarVersion) console.log('  jar version:', chalk.cyan(jar.jarVersion), latestVersion === jar.jarVersion ? chalk.green('(latest)') : chalk.red('(outdated)'))
	}

	if (args.profile) process.chdir('../..')
}