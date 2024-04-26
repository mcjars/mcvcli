import fs from "fs"
import path from "path"
import chalk from "chalk"
import init from "src/commands/init"
import getConfig from "src/utils/config"
import cleanPath from "src/utils/cleanPath"

export type Args = {
	name: string
}

export default async function profileCreate(args: Args) {
	args.name = cleanPath(args.name)

	const config = getConfig()

	if (args.name === config.data.profileName) {
		console.log('cannot create profile with same name as current profile!')
		process.exit(1)
	}

	if (fs.existsSync(path.join('.mcvcli.profiles', args.name, '.mcvcli.json'))) {
		console.log('profile already exists!')
		process.exit(1)
	}

	await fs.promises.mkdir(path.join('.mcvcli.profiles', args.name), { recursive: true })
	process.chdir(path.join('.mcvcli.profiles', args.name))

	await init({ directory: '.' }, args.name)

	process.chdir('../..')
	console.log('profile created! switch to it using', chalk.cyan(`mcvcli profile use ${args.name}`))
}