import fs from "fs"
import chalk from "chalk"

import { configSchema } from "src/types/config"
import { z } from "zod"

export class Config {
	constructor(public data: z.infer<typeof configSchema>) {
		this.data = data
	}

	public write() {
		fs.writeFileSync('.mccli.json', JSON.stringify(this.data, null, 2))
	}
}

export default function getConfig() {
	if (!fs.existsSync('.mccli.json')) {
		console.log('no', chalk.yellow('.mccli.json'), 'file found!')
		console.log('initialize using', chalk.cyan('mccli init .'))

		process.exit(1)
	}

	try {
		const config = configSchema.parse(JSON.parse(fs.readFileSync('.mccli.json', 'utf-8')))

		return new Config(config)
	} catch {
		console.log('invalid', chalk.yellow('.mccli.json'), 'file!')

		process.exit(1)
	}
}