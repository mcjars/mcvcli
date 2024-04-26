import fs from "fs"
import path from "path"
import cleanPath from "src/utils/cleanPath"
import { filesystem } from "@rjweb/utils"
import bytes from "bytes"
import * as tar from "tar"

export type Args = {
	name: string
	clean: boolean
}

export default async function backupRestore(args: Args) {
	args.name = cleanPath(args.name)

	if (!fs.existsSync(path.join('.mcvcli.backups', `${args.name}.tar.gz`)) && !fs.existsSync(path.join('.mcvcli.backups', args.name, '.mcvcli.json'))) {
		console.log('backup not found!')
		process.exit(1)
	}

	if (args.clean) {
		fs.readdirSync('.')
			.filter((file) => !file.includes('.mcvcli.profiles') && !file.includes('.mcvcli.backups'))
			.forEach((file) => fs.rmSync(file, { recursive: true }))
	}

	switch (true) {
		case fs.existsSync(path.join('.mcvcli.backups', args.name, '.mcvcli.json')): {
			await fs.promises.mkdir(path.join('.mcvcli.backups', args.name), { recursive: true })

			console.log('restoring backup...')

			let files = 0, size = 0
			for (const file of filesystem.walk(path.join('.mcvcli.backups', args.name), { recursive: true })) {
				if (!file.isFile()) continue

				files++
				size += fs.statSync(path.join(file.path, file.name)).size
			}

			for (const file of filesystem.walk(path.join('.mcvcli.backups', args.name), { recursive: true })) {
				if (!file.isFile()) continue
				const relative = path.relative(path.join('.mcvcli.backups', args.name), path.join(file.path, file.name))

				fs.mkdirSync(path.dirname(relative), { recursive: true })
				fs.copyFileSync(path.join(file.path, file.name), relative)
				process.stdout.write(`\rrestoring backup... ${files--} files left      `)
			}

			console.log(`\n(${bytes(size, { decimalPlaces: 2 })}) backup restored!`)
			break
		}

		case fs.existsSync(path.join('.mcvcli.backups', `${args.name}.tar.gz`)): {
			await fs.promises.mkdir('.mcvcli.backups', { recursive: true })

			console.log('restoring backup...')

			const readStream = fs.createReadStream(path.join('.mcvcli.backups', `${args.name}.tar.gz`))

			readStream.pipe(tar.x({
				gzip: true,
				cwd: '.'
			}) as any)

			readStream.on('data', () => {
				process.stdout.write(`\rrestoring backup... ${bytes(readStream.bytesRead, { decimalPlaces: 2 })} read      `)
			})

			await new Promise((resolve) => {
				readStream.once('end', resolve)
			})

			readStream.destroy()
			
			console.log(`\n(${bytes(readStream.bytesRead, { decimalPlaces: 2 })}) backup restored!`)
			break
		}
	}
}