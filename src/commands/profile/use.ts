import fs from "fs"
import path from "path"
import chalk from "chalk"
import getConfig from "src/utils/config"

export type Args = {
	profile: string
}

export default async function profileUse(args: Args) {
	const config = getConfig()

	if (args.profile === config.profileName) {
		console.log('already using this profile!')
		process.exit(1)
	}

	if (!fs.existsSync(path.join('.mccli.profiles', args.profile)) || !fs.existsSync(path.join('.mccli.profiles', args.profile, '.mccli.json'))) {
		console.log('profile not found or invalid!')
		process.exit(1)
	}

	if (fs.existsSync(path.join('.mccli.profiles', config.profileName, '.mccli.json'))) {
		console.log('profile folder is not empty!')
		process.exit(1)
	}

	await fs.promises.mkdir(path.join('.mccli.profiles', config.profileName), { recursive: true })
	
	const files = fs.readdirSync('.').filter((file) => !file.startsWith('.mccli.profiles'))
	files.forEach((file) => fs.renameSync(file, path.join('.mccli.profiles', config.profileName, file)))

	const profileFiles = fs.readdirSync(path.join('.mccli.profiles', args.profile))
	profileFiles.forEach((file) => fs.renameSync(path.join('.mccli.profiles', args.profile, file), file))

	console.log('switched to profile', chalk.cyan(args.profile))
}