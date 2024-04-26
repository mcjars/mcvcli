import chalk from "chalk"
import fs from "fs"
import path from "path"
import getConfig from "src/utils/config"

export type Args = {}

export default async function profileList(args: Args) {
	const config = getConfig(),
		profiles = new Set((await fs.promises.readdir('.mcvcli.profiles').catch(() => []))
			.filter((profile) => fs.existsSync(path.join('.mcvcli.profiles', profile, '.mcvcli.json'))))

	profiles.add(config.data.profileName)

	console.log('profiles:')
	profiles.forEach((profile) => console.log('  - ', profile))
	if (!profiles.size) console.log('  (none)')

	console.log('current profile:', chalk.cyan(config.data.profileName))
}