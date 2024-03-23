import chalk from "chalk"
import fs from "fs"
import getConfig from "src/utils/config"

export type Args = {}

export default async function profileList(args: Args) {
	const config = getConfig(),
		profiles = await fs.promises.readdir('.mccli.profiles').catch(() => [])

	console.log('profiles:')
	profiles.forEach((profile) => console.log('  - ', profile))
	if (!profiles.length) console.log('  (none)')

	console.log('current profile:', chalk.cyan(config.profileName))
}