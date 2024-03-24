import getConfig from "src/utils/config"
import fs from "fs"
import getCache from "src/utils/cache"
import { getMods } from "src/utils/mods"
import chalk from "chalk"
import path from "path"

export type Args = {}

export default async function mods(args: Args) {
	const config = getConfig(),
		cache = getCache()

	if (!fs.existsSync(path.join(path.dirname(config.data.jarFile), 'mods'))) {
		console.log('no mods folder found!')
		process.exit(1)
	}

	const mods = await getMods(path.join(path.dirname(config.data.jarFile), 'mods'), cache)

	console.log('mods:')
	for (const mod of mods) {
		if ('infos' in mod) {
			console.log('  -', mod.infos.title)
			console.log('    version:', chalk.cyan(mod.infos.version))
			console.log('    license:', chalk.cyan(mod.infos.project.license))
			console.log(mod.infos.latest ? chalk.green('    (latest)') : chalk.red('    (outdated)'))
		}
	}

	if (!mods.length) console.log('  (none)')
	if (mods.some((mod) => !('infos' in mod))) console.log(`  (${mods.filter((mod) => !('infos' in mod)).length} mods are missing infos)`)
	if (mods.some((mod) => 'infos' in mod && !mod.infos.latest)) console.log(`  (${mods.filter((mod) => 'infos' in mod && !mod.infos.latest).length} mods are outdated)`)
	if (mods.length) console.log('total:', chalk.cyan(mods.length))
}