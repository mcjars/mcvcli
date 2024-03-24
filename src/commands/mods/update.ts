import fs from "fs"
import path from "path"
import getCache from "src/utils/cache"
import getConfig from "src/utils/config"
import { getMods } from "src/utils/mods"
import enquirer from "enquirer"
import chalk from "chalk"
import * as api from "src/api"
import getJarVersion from "src/utils/jar"
import download from "src/utils/download"

export type Args = {}

export default async function modsUpdate(args: Args) {
	const config = getConfig(),
		cache = getCache(),
		jar = await getJarVersion(config.data.jarFile)

	if (!fs.existsSync(path.join(path.dirname(config.data.jarFile), 'mods'))) {
		console.log('no mods folder found!')
		process.exit(1)
	}

	const mods = await getMods(path.join(path.dirname(config.data.jarFile), 'mods'), jar, cache).then((mods) => mods.filter((mod) => 'infos' in mod)),
		modsWithUpdate = mods.filter((mod) => 'infos' in mod && mod.latest?.version_number !== mod.infos.version)

	if (!mods.length) {
		console.log('no mods found!')
		process.exit(1)
	}

	if (!modsWithUpdate.length) {
		console.log('all mods are up to date!')
		process.exit(1)
	}

	const { updateAll } = await enquirer.prompt<{
		updateAll: boolean
	}>({
		type: 'confirm',
		message: `Update All Mods? ${chalk.red('THIS MAY BREAK YOUR SERVER!')}`,
		name: 'updateAll'
	})

	if (updateAll) {
		console.log('updating all mods...')
		for (const mod of modsWithUpdate) {
			if (!('infos' in mod) || !mod.latest) continue

			const file = mod.latest.files.find((file) => file.primary) ?? mod.latest.files[0]

			await fs.promises.rm(path.join(path.dirname(config.data.jarFile), 'mods', mod.file))
			await download(file.filename, file.url, path.join(path.dirname(config.data.jarFile), 'mods', file.filename))
		}

		console.log('finished updating all mods!')
		process.exit(0)
	} else {
		const { selectedMods } = await enquirer.prompt<{
			selectedMods: string[]
		}>({
			type: 'multiselect',
			message: 'Mods to Update',
			name: 'selectedMods',
			choices: modsWithUpdate.map((mod: any) => ({ message: mod.infos.project.title, value: mod.infos.project.id })) as any,
			// @ts-ignore
			limit: 10
		})

		console.log('updating selected mods...')
		for (const mod of modsWithUpdate) {
			if (!('infos' in mod) || !selectedMods.includes(mod.infos.project.id) || !mod.latest) continue

			const file = mod.latest.files.find((file) => file.primary) ?? mod.latest.files[0]

			await fs.promises.rm(path.join(path.dirname(config.data.jarFile), 'mods', mod.file))
			await download(file.filename, file.url, path.join(path.dirname(config.data.jarFile), 'mods', file.filename))
		}

		console.log('finished updating selected mods!')
		process.exit(0)
	}
}