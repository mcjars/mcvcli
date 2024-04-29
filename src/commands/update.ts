import chalk from "chalk"
import path from "path"
import enquirer from "enquirer"
import * as api from "src/api"
import getConfig from "src/utils/config"
import getJarVersion from "src/utils/jar"
import getCache from "src/utils/cache"

export type Args = {}

export default async function update(args: Args) {
	console.log('checking currently installed version...')

	const config = getConfig(),
		cache = getCache(),
		version = await getJarVersion(path.resolve(config.data.jarFile), cache)

	const [ { latestJar, latestMc }, { latestVersion }, modpackVersions ] = await Promise.all([
		api.latest(version.type, version.minecraftVersion!),
		api.latestModpack(config.data.modpackSlug ?? 'ballspack-2000'),
		api.modpackVersions(config.data.modpackSlug ?? 'ballspack-2000')
	])

	const modpackVersion = modpackVersions.find((v) => v.id === config.data.modpackVersion)

	if (config.data.modpackSlug && config.data.modpackVersion) {
		const infos = await api.modpackInfos(config.data.modpackSlug)

		console.log('installed modpack:')
		console.log('  title:', chalk.cyan(infos.title))
		console.log('  license:', chalk.cyan(infos.license.id))
		console.log('  version:', chalk.cyan(modpackVersion?.version_number ?? 'unknown'), latestVersion === modpackVersion?.version_number ? chalk.green('(latest)') : chalk.red('(outdated)'))
	}

	console.log('currently installed jar location:', chalk.cyan(config.data.jarFile))
	console.log('currently installed jar version:')
	console.log('  type:', chalk.cyan(version.type))
	if (version.minecraftVersion) console.log('  minecraft version:', chalk.cyan(version.minecraftVersion), latestMc === version.minecraftVersion ? chalk.green('(latest)') : chalk.red('(outdated)'))
	if (version.jarVersion) console.log('  jar version:', chalk.cyan(version.jarVersion), latestJar === version.jarVersion ? chalk.green('(latest)') : chalk.red('(outdated)'))

	if (!config.data.modpackSlug && version.type === 'unknown') {
		console.log('server type is unknown, unable to update!')
		console.log('use', chalk.cyan('mcvcli install'), 'to install a new version.')
		process.exit(1)
	}

	if (config.data.modpackSlug && latestVersion === modpackVersion?.version_number) {
		console.log('server is already up to date!')
		console.log('use', chalk.cyan('mcvcli install'), 'to install a new version.')
		process.exit(0)
	}

	if (!config.data.modpackSlug && latestJar === version.jarVersion && latestMc === version.minecraftVersion) {
		console.log('server is already up to date!')
		console.log('use', chalk.cyan('mcvcli install'), 'to install a new version.')
		process.exit(0)
	}

	const { update } = await enquirer.prompt<{
		update: 'Minecraft Version' | 'Jar Build' | 'Modpack Version'
	}>({
		type: 'select',
		message: 'Update',
		name: 'update',
		choices: [
			config.data.modpackSlug && latestVersion !== modpackVersion?.version_number && 'Modpack Version',
			...!config.data.modpackSlug ? [
				latestMc !== version.minecraftVersion && 'Minecraft Version',
				latestJar !== version.jarVersion && 'Jar Build'
			] : []
		].filter(Boolean) as string[]
	})

	switch (update) {
		case "Modpack Version": {
			const versions = modpackVersions,
				index = versions.findIndex((v) => v.id === config.data.modpackVersion)

			const { version } = await enquirer.prompt<{
				version: string
			}>({
				type: 'select',
				message: 'Modpack Version',
				name: 'version',
				choices: versions.filter((_, i) => i < index).map((v) => v.title),
				// @ts-ignore
				limit: 10
			})

			const modpackVersion = modpackVersions.find((v) => v.title === version)

			await api.installModpack(config.data.modpackSlug!, config.data.modpackVersion, modpackVersion!.id, config)
			config.write()

			break
		}

		case "Minecraft Version": {
			if (version.type === 'unknown') process.exit(1)

			const versions = await api.versions(version.type),
				index = versions.indexOf(version.minecraftVersion!)

			const { serverVersion } = await enquirer.prompt<{
				serverVersion: string
			}>({
				type: 'select',
				message: 'Server Version',
				name: 'serverVersion',
				choices: versions.filter((_, i) => i > index).reverse(),
				// @ts-ignore
				limit: 10
			})

			const builds = await api.builds(version.type, serverVersion),
				javaVersions = await api.adoptium.versions(),
				latest = builds[0]

			const { javaVersion } = await enquirer.prompt<{
				javaVersion: string
			}>({
				type: 'autocomplete',
				message: 'Java Version',
				name: 'javaVersion',
				choices: javaVersions.map((version) => version.toString()),
				// @ts-ignore
				limit: 5
			})

			config.data.javaVersion = parseInt(javaVersion)
			config.write()

			await api.install(latest.download, config)
			break
		}

		case "Jar Build": {
			if (version.type === 'unknown') process.exit(1)

			const builds = await api.builds(version.type, version.minecraftVersion!),
				latest = builds[0]

			await api.install(latest.download, config)
			break
		}
	}

	console.log('server updated!')
}