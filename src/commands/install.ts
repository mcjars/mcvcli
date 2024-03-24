import enquirer from "enquirer"
import * as api from "../api"
import getConfig from "../utils/config"
import chalk from "chalk"

export type Args = {}

export default async function install(args: Args) {
	const config = getConfig()

	const { install } = await enquirer.prompt<{
		install: 'Install Jar' | 'Install Modpack'
	}>({
		type: 'select',
		message: 'Install Type',
		name: 'install',
		choices: [
			'Install Jar',
			'Install Modpack'
		]
	})

	switch (install) {
		case "Install Jar": {
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
		
			await api.install(latest.download, config)
		
			console.log('server installed!')
		}

		case "Install Modpack": {
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

			const data = await api.modpackInfos(modpackSlug)

			console.log('modpack found:')
			console.log('  title:', chalk.cyan(data.title))
			console.log('  license:', chalk.cyan(data.license.id))

			const modpackVersions = await api.modpackVersions(modpackSlug)

			if (modpackVersions.length === 0) {
				console.log('modpack not found!')
				process.exit(1)
			}

			const { version } = await enquirer.prompt<{
				version: string
			}>({
				type: 'select',
				message: 'Modpack Version',
				name: 'version',
				choices: modpackVersions.map((v) => v.title)
			})

			const modpackVersion = modpackVersions.find((v) => v.title === version)

			await api.installModpack(modpackSlug, config.data.modpackVersion, modpackVersion!.id, config)
			config.data.modpackSlug = modpackSlug
			config.write()

			console.log('server installed!')
		}
	}
}