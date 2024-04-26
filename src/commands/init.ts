import fs from "fs"
import path from "path"
import enquirer from "enquirer"
import chalk from "chalk"
import * as api from "src/api"
import getJarVersion from "src/utils/jar"
import { Config } from "src/utils/config"
import getCache from "src/utils/cache"

export type Args = {
	directory: string
}

export default async function init(args: Args, profileName?: string) {
	if (!fs.existsSync(args.directory)) {
		fs.mkdirSync(args.directory)
	}

	const previousCwd = process.cwd()
	process.chdir(args.directory)

	if (fs.existsSync('.mcvcli.json')) {
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
			'Install New (Jar)',
			'Install New (Modpack)',
			...files.filter((file) => file.endsWith('.jar'))
		]
	})

	switch (jarFile) {
		case "Install New (Jar)": {
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
				javaVersions = await api.adoptium.versions(),
				latest = builds[0]

			const { ramMB, javaVersion } = await enquirer.prompt<{
				ramMB: number
				javaVersion: string
			}>([
				{
					type: 'numeral',
					message: 'Server RAM (MB)',
					name: 'ramMB',
					min: 1024,
					initial: 4096
				},
				{
					type: 'autocomplete',
					message: 'Java Version',
					name: 'javaVersion',
					choices: javaVersions.map((version) => version.toString()),
					// @ts-ignore
					limit: 5
				}
			])

			const config = new Config({
				configVersion: 3,
				__README: 'This file is used to store the configuration for the mcvcli tool. Do not modify this file unless you know what you are doing.',
				jarFile: 'server.jar',
				javaVersion: parseInt(javaVersion),
				profileName: profileName ?? 'default',
				modpackSlug: null,
				modpackVersion: null,
				ramMB
			})

			await api.install(latest.download, config)
			config.write()

			break
		}

		case "Install New (Modpack)": {
			const initialPacks = await api.searchModpacks('')

			const { modpackSlug } = await enquirer.prompt<{
				modpackSlug: string
			}>({
				type: 'autocomplete',
				message: 'Modrinth Modpack',
				name: 'modpackSlug',
				choices: initialPacks.map((pack) => ({ name: pack.title, value: pack.slug })),
				// @ts-ignore
				async suggest(input: string) {
					const packs = await api.searchModpacks(input)
					return packs.map((pack) => ({ message: pack.title, value: pack.slug }))
				}
			})

			const data = await api.modpackInfos(modpackSlug),
				javaVersions = await api.adoptium.versions()

			console.log('modpack found:')
			console.log('  title:', chalk.cyan(data.title))
			console.log('  license:', chalk.cyan(data.license.id))

			const { ramMB, javaVersion } = await enquirer.prompt<{
				ramMB: number
				javaVersion: string
			}>([
				{
					type: 'numeral',
					message: 'Server RAM (MB)',
					name: 'ramMB',
					min: 1024,
					initial: 4096
				},
				{
					type: 'autocomplete',
					message: 'Java Version',
					name: 'javaVersion',
					choices: javaVersions.map((version) => version.toString()),
					// @ts-ignore
					limit: 5
				}
			])

			const config = new Config({
				configVersion: 3,
				__README: 'This file is used to store the configuration for the mcvcli tool. Do not modify this file unless you know what you are doing.',
				jarFile: 'server.jar',
				javaVersion: parseInt(javaVersion),
				profileName: profileName ?? 'default',
				modpackSlug,
				modpackVersion: null,
				ramMB
			})

			const versions = await api.modpackVersions(modpackSlug)

			if (versions.length === 0) {
				console.log('no versions found!')
				process.exit(1)
			}

			const { version } = await enquirer.prompt<{
				version: string
			}>({
				type: 'autocomplete',
				message: 'Modpack Version',
				name: 'version',
				choices: versions.map((v) => v.title),
				// @ts-ignore
				limit: 10
			})

			const modpackVersion = versions.find((v) => v.title === version)

			await api.installModpack(modpackSlug, null, modpackVersion!.id, config)
			config.write()

			break
		}

		default: {
			console.log('checking installed version...')
			const cache = getCache()

			const version = await getJarVersion(path.resolve(jarFile), cache)

			const { latestJar, latestMc } = await api.latest(version.type, version.minecraftVersion!),
				javaVersions = await api.adoptium.versions()

			console.log('installed jar location:', chalk.cyan(jarFile))
			console.log('installed jar version:')
			console.log('  type:', chalk.cyan(version.type))
			if (version.minecraftVersion) console.log('  minecraft version:', chalk.cyan(version.minecraftVersion), latestMc === version.minecraftVersion ? chalk.green('(latest)') : chalk.red('(outdated)'))
			if (version.jarVersion) console.log('  jar version:', chalk.cyan(version.jarVersion), latestJar === version.jarVersion ? chalk.green('(latest)') : chalk.red('(outdated)'))

			const { ramMB, javaVersion } = await enquirer.prompt<{
				ramMB: number
				javaVersion: string
			}>([
				{
					type: 'numeral',
					message: 'Server RAM (MB)',
					name: 'ramMB',
					min: 1024,
					initial: 4096
				},
				{
					type: 'autocomplete',
					message: 'Java Version',
					name: 'javaVersion',
					choices: javaVersions.map((version) => version.toString()),
					// @ts-ignore
					limit: 5
				}
			])

			const config = new Config({
				configVersion: 3,
				__README: 'This file is used to store the configuration for the mcvcli tool. Do not modify this file unless you know what you are doing.',
				jarFile,
				javaVersion: parseInt(javaVersion),
				profileName: profileName ?? 'default',
				modpackSlug: null,
				modpackVersion: null,
				ramMB
			})

			config.write()

			break
		}
	}

	if (!profileName) fs.mkdirSync(path.join('.mcvcli.profiles', 'default'), { recursive: true })
	process.chdir(previousCwd)
	console.log('mccli initialized!')
}