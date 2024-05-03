import fs from "fs"
import path from "path"
import chalk from "chalk"
import * as api from "src/api"
import getConfig from "src/utils/config"
import getJarVersion from "src/utils/jar"
import getCache from "src/utils/cache"

export type Args = {
	profile?: string
}

export default async function version(args: Args) {
	if (args.profile) {
		if (!fs.existsSync(path.join('.mcvcli.profiles', args.profile))) {
			console.log('profile not found!')
			process.exit(1)
		}

		process.chdir(path.join('.mcvcli.profiles', args.profile))
	}

	console.log('checking installed version...')
	const start = performance.now()

	const config = getConfig(),
		cache = getCache()

	if (!config.data.modpackSlug && !config.data.modpackVersion) {
		const version = await getJarVersion(path.resolve(config.data.jarFile), cache),
			{ latestJar, latestMc } = await api.latest(version.type, version.minecraftVersion!)

		console.log('checking installed version... done', chalk.gray(`(${(performance.now() - start).toFixed(2)}ms)`), '\n')

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
			getJarVersion(path.resolve(config.data.jarFile), cache)
		])

		const version = versions.find(v => v.id === modpackVersion),
			latestJar = await api.latest(jar.type, jar.minecraftVersion!)

		console.log('checking installed version... done', chalk.gray(`(${(performance.now() - start).toFixed(2)}ms)`), '\n')

		console.log('installed modpack:')
		console.log('  title:', chalk.cyan(infos.title))
		console.log('  license:', chalk.cyan(infos.license.id))
		console.log('  version:', chalk.cyan(version?.version_number ?? 'unknown'), latestVersion === version?.version_number ? chalk.green('(latest)') : chalk.red('(outdated)'))
		console.log('installed jar location:', chalk.cyan(config.data.jarFile))
		console.log('installed jar version:')
		console.log('  type:', chalk.cyan(jar.type))
		if (jar.minecraftVersion) console.log('  minecraft version:', chalk.cyan(jar.minecraftVersion), latestJar.latestMc === jar.minecraftVersion ? chalk.green('(latest)') : chalk.red('(outdated)'))
		if (jar.jarVersion) console.log('  jar version:', chalk.cyan(jar.jarVersion), latestJar.latestJar === jar.jarVersion ? chalk.green('(latest)') : chalk.red('(outdated)'))
	}

	console.log('installed java version:', chalk.cyan(config.data.javaVersion))

	if (args.profile) process.chdir('../..')
}