import fs from "fs"
import path from "path"
import getCache from "src/utils/cache"
import getConfig from "src/utils/config"
import { getMods } from "src/utils/mods"
import enquirer from "enquirer"
import chalk from "chalk"
import getJarVersion from "src/utils/jar"

export type Args = {}

export default async function modsUninstall(args: Args) {
	const config = getConfig(),
		cache = getCache(),
		jar = await getJarVersion(config.data.jarFile, cache)

	if (!fs.existsSync(path.join(path.dirname(config.data.jarFile), 'mods'))) {
		console.log('no mods folder found!')
		process.exit(1)
	}

	const mods = await getMods(path.join(path.dirname(config.data.jarFile), 'mods'), jar, cache).then((mods) => mods.filter((mod) => 'infos' in mod))

	if (!mods.length) {
		console.log('no mods found!')
		process.exit(1)
	}

	const { selectedMods } = await enquirer.prompt<{
		selectedMods: string[]
	}>({
		type: 'multiselect',
		message: 'Mods to Uninstall',
		name: 'selectedMods',
		choices: mods.map((mod: any) => ({ message: mod.infos.project.title, value: mod.file })) as any,
		// @ts-ignore
		limit: 10
	})

	console.log('uninstalling mods...')
	for (const mod of selectedMods) {
		await fs.promises.rm(path.join(path.dirname(config.data.jarFile), 'mods', mod))
		console.log('uninstalled', chalk.cyan(`mods/${mod}`))
	}

	console.log('finished uninstalling mods!')
}