import { filesystem } from "@rjweb/utils"
import bytes from "bytes"
import chalk from "chalk"
import fs from "fs"
import path from "path"

export type Args = {}

export default async function backupList(args: Args) {
	const backups: [name: string, type: string, size: number][] = []

	if (fs.existsSync('.mcvcli.backups')) for (const file of filesystem.walk('.mcvcli.backups')) {
		if (file.isFile() && file.name.endsWith('.tar.gz')) {
			backups.push([file.name.slice(0, -7), 'tar', fs.statSync(path.join(file.path, file.name)).size])
		} else {
			let size = 0

			for (const backupFile of filesystem.walk(path.join(file.path, file.name), { recursive: true })) {
				if (backupFile.isFile()) size += fs.statSync(path.join(backupFile.path, backupFile.name)).size
			}

			backups.push([file.name, 'folder', size])
		}
	}

	console.log('backups:')
	for (const backup of backups) {
		console.log('  -', backup[0])
		console.log('    type:', chalk.cyan(backup[1]))
		console.log('    size:', chalk.cyan(bytes(backup[2], { decimalPlaces: 2 })))
	}

	if (!backups.length) console.log('  (none)')
	if (backups.length) {
		console.log('total:')
		console.log('  backups:', chalk.cyan(backups.length))
		console.log('  size:', chalk.cyan(bytes(backups.reduce((acc, backup) => acc + backup[2], 0), { decimalPlaces: 2 })))
	}
}