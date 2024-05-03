import fs from "fs"
import chalk from "chalk"
import { configSchema, configVersions } from "src/types/config"
import { z } from "zod"

function upgradeConfig(config: any) {
	const previousVersion = config.configVersion

	switch (config.configVersion) {
		case undefined: {
			config = {
				configVersion: 2,
				__README: 'This file is used to store the configuration for the mcvcli tool. Do not modify this file unless you know what you are doing.',
				jarFile: config.jarFile,
				profileName: config.profileName,
				modpackSlug: null,
				modpackVersion: null,
				ramMB: config.ramMB
			}

			break
		}

		case 2: {
			config.configVersion = 3
			config.javaVersion = 21

			break
		}
	}

	if (config.configVersion !== previousVersion) {
		console.log('upgraded config to version', chalk.cyan(config.configVersion))
		fs.writeFileSync('.mcvcli.json', JSON.stringify(config, null, 2))
	}

	if (config.configVersion !== configVersions._def.options.at(-1)?.value) return upgradeConfig(config)
	else return config
}

export class Config {
	constructor(public data: z.infer<typeof configSchema>) {}

	public write() {
		fs.writeFileSync('.mcvcli.json', JSON.stringify(this.data, null, 2))
	}
}

export default function getConfig(profile?: string) {
	const file = profile ? `.mcvcli.profiles/${profile}/.mcvcli.json` : '.mcvcli.json'

	if (!fs.existsSync(file)) {
		console.log('no', chalk.yellow('.mcvcli.json'), 'file found!')
		console.log('initialize using', chalk.cyan('mcvcli init .'))

		process.exit(1)
	}

	try {
		const config = JSON.parse(fs.readFileSync(file, 'utf-8'))

		return new Config(configSchema.parse(upgradeConfig(config)))
	} catch {
		console.log('invalid', chalk.yellow('.mcvcli.json'), 'file!')

		process.exit(1)
	}
}