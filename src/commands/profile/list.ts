import chalk from "chalk"
import fs from "fs"
import path from "path"
import getCache from "src/utils/cache"
import getConfig from "src/utils/config"
import jarVersion from "src/utils/jar"

export type Args = {}

export default async function profileList(args: Args) {
	const config = getConfig(),
		cache = getCache(),
		profiles = new Set((await fs.promises.readdir('.mcvcli.profiles').catch(() => []))
			.filter((profile) => fs.existsSync(path.join('.mcvcli.profiles', profile, '.mcvcli.json'))))

	profiles.add(config.data.profileName)

	const profileVersions: Record<string, string> = {},
		start = performance.now()

	console.log('checking versions...')

	const data = await Promise.all([...profiles].map(async(profile) => {
		const profileConfig = getConfig(profile === config.data.profileName ? undefined : profile),
			version = await jarVersion(profile === config.data.profileName ? profileConfig.data.jarFile : path.join('.mcvcli.profiles', profile, profileConfig.data.jarFile), cache)

		return version.type === 'unknown' ? 'unknown' : `${version.type} ${version.minecraftVersion}`
	}))

	for (let i = 0; i < profiles.size; i++) profileVersions[[...profiles][i]] = data[i]

	console.log('checking versions... done', chalk.gray(`(${(performance.now() - start).toFixed(2)}ms)`), '\n')

	console.log('profiles:')
	if (!profiles.size) console.log('  (none)')

	for (const profile of profiles) {
		console.log('  ', profile === config.data.profileName ? chalk.cyan(profile) : profile, chalk.gray(`(${profileVersions[profile]})`))
	}

	console.log('current profile:', chalk.cyan(config.data.profileName))
}