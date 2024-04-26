import fs from "fs"
import path from "path"
import chalk from "chalk"
import cleanPath from "src/utils/cleanPath"
import { filesystem } from "@rjweb/utils"
import bytes from "bytes"
import * as tar from "tar"

export type Args = {
	name: string
	method: string
	override: boolean
}

export default async function backupCreate(args: Args) {
	args.name = cleanPath(args.name)

	if (!args.override && fs.existsSync(path.join('.mcvcli.backups', `${args.name}.tar.gz`)) || fs.existsSync(path.join('.mcvcli.backups', args.name, '.mcvcli.json'))) {
		console.log('backup already exists!')
		process.exit(1)
	}

	if (args.override) {
		if (fs.existsSync(path.join('.mcvcli.backups', `${args.name}.tar.gz`))) fs.unlinkSync(path.join('.mcvcli.backups', `${args.name}.tar.gz`))
		if (fs.existsSync(path.join('.mcvcli.backups', args.name))) await fs.promises.rm(path.join('.mcvcli.backups', args.name), { recursive: true })
	}

	switch (args.method) {
		case "folder": {
			await fs.promises.mkdir(path.join('.mcvcli.backups', args.name), { recursive: true })

			console.log('creating backup...')

			let files = 0, size = 0
			for (const file of filesystem.walk('.', { recursive: true })) {
				if (!file.isFile() || file.path.includes('.mcvcli.backups') || file.path.includes('.mcvcli.profiles')) continue

				files++
				size += fs.statSync(path.join(file.path, file.name)).size
			}

			for (const file of filesystem.walk('.', { recursive: true })) {
				if (!file.isFile() || file.path.includes('.mcvcli.backups') || file.path.includes('.mcvcli.profiles')) continue
				const relative = path.relative('.', path.join(file.path, file.name))

				fs.mkdirSync(path.join('.mcvcli.backups', args.name, path.dirname(relative)), { recursive: true })
				fs.copyFileSync(path.join(file.path, file.name), path.join('.mcvcli.backups', args.name, relative))
				process.stdout.write(`\rcreating backup... ${files--} files left      `)
			}

			console.log(`\n(${bytes(size, { decimalPlaces: 2 })}) backup created!`, chalk.cyan(`mcvcli backup restore ${args.name}`))
			break
		}

		case "tar": {
			await fs.promises.mkdir('.mcvcli.backups', { recursive: true })

			console.log('creating backup...')

			const stream = tar.c({
				gzip: true,
				cwd: '.',
				filter: (path) => !path.includes('.mcvcli.backups') && !path.includes('.mcvcli.profiles')
			}, ['.'])

			const writeStream = fs.createWriteStream(path.join('.mcvcli.backups', `${args.name}.tar.gz`))

			stream.on('data', (chunk) => {
				process.stdout.write(`\rcreating backup... ${bytes(writeStream.bytesWritten, { decimalPlaces: 2 })} written      `)

				stream.pause()
				writeStream.write(chunk, () => stream.resume())
			})

			await new Promise((resolve) => {
				stream.once('end', resolve)
			})

			writeStream.end()
			
			console.log(`\n(${bytes(writeStream.bytesWritten, { decimalPlaces: 2 })}) backup created!`, chalk.cyan(`mcvcli backup restore ${args.name}`))
			break
		}
	}
}