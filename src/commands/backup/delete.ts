import fs from "fs"
import path from "path"
import cleanPath from "src/utils/cleanPath"

export type Args = {
	name: string
}

export default async function backupDelete(args: Args) {
	args.name = cleanPath(args.name)

	if (!fs.existsSync(path.join('.mcvcli.backups', `${args.name}.tar.gz`)) && !fs.existsSync(path.join('.mcvcli.backups', args.name, '.mcvcli.json'))) {
		console.log('backup not found!')
		process.exit(1)
	}

	if (fs.existsSync(path.join('.mcvcli.backups', `${args.name}.tar.gz`))) {
		fs.rmSync(path.join('.mcvcli.backups', `${args.name}.tar.gz`))
		console.log('backup deleted!')
	} else {
		fs.rmSync(path.join('.mcvcli.backups', args.name), { recursive: true })
		console.log('backup deleted!')
	}
}