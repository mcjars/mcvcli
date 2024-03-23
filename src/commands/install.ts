import enquirer from "enquirer"
import * as api from "../api"
import getConfig from "../utils/config"

export type Args = {}

export default async function install(args: Args) {
	const config = getConfig()

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