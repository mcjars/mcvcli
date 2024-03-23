import chalk from "chalk"
import path from "path"
import enquirer from "enquirer"
import * as api from "src/api"
import getConfig from "src/utils/config"
import getJarVersion from "src/utils/jar"

export type Args = {}

export default async function update(args: Args) {
	console.log('checking currently installed version...')

	const config = getConfig(),
		version = await getJarVersion(path.resolve(config.data.jarFile))

	const { latestJar, latestMc } = await api.latest(version.type, version.minecraftVersion!)

	console.log('currently installed jar location:', chalk.cyan(config.data.jarFile))
	console.log('currently installed jar version:')
	console.log('type:', chalk.cyan(version.type))
	if (version.minecraftVersion) console.log('minecraft version:', chalk.cyan(version.minecraftVersion), latestMc === version.minecraftVersion ? chalk.green('(latest)') : chalk.red('(outdated)'))
	if (version.jarVersion) console.log('jar version:', chalk.cyan(version.jarVersion), latestJar === version.jarVersion ? chalk.green('(latest)') : chalk.red('(outdated)'))

	if (version.type === 'unknown') {
		console.log('server type is unknown, unable to update!')
		console.log('use', chalk.cyan('mccli install'), 'to install a new version.')
		process.exit(1)
	}

	if (latestJar === version.jarVersion && latestMc === version.minecraftVersion) {
		console.log('server is already up to date!')
		console.log('use', chalk.cyan('mccli install'), 'to install a new version.')
		process.exit(0)
	}

	const { update } = await enquirer.prompt<{
		update: 'Minecraft Version' | 'Jar Build'
	}>({
		type: 'select',
		message: 'Update',
		name: 'update',
		choices: [
			latestMc !== version.minecraftVersion && 'Minecraft Version',
			latestJar !== version.jarVersion && 'Jar Build'
		].filter(Boolean) as string[]
	})

	switch (update) {
		case "Minecraft Version": {
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
				latest = builds[0]

			await api.install(latest.download, config)
			break
		}

		case "Jar Build": {
			const builds = await api.builds(version.type, version.minecraftVersion!),
				latest = builds[0]

			await api.install(latest.download, config)
			break
		}
	}

	console.log('server updated!')
}