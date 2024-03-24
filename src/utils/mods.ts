import { filesystem, number } from "@rjweb/utils"
import chalk from "chalk"
import fs from "fs"
import path from "path"
import { Cache } from "src/utils/cache"
import * as api from "src/api"
import readline from "readline"

export type Mod = {
	file: string
} | {
	file: string
	infos: Awaited<ReturnType<typeof api['modrinth']['projectByHash']>>
}

export async function getMods(directory: string, cache: Cache) {
	const files = await fs.promises.readdir(directory)

	let progress = 0

	const mods: Mod[] = []
	for (const file of files) {
		progress++

		readline.moveCursor(process.stdout, 0, -1)
		process.stdout.write(`\rchecking mod ${chalk.cyan(file)}                  \n${number.round((progress / files.length) * 100, 2)}% (${progress} / ${files.length})   `)

		const hash = await filesystem.hash(path.join(directory, file), { algorithm: 'sha1' })

		const mod = cache.getString(`mods_${hash}`)
		if (mod) {
			try {
				const data = JSON.parse(mod) as Awaited<ReturnType<typeof api['modrinth']['projectByHash']>>

				mods.push({
					file,
					infos: data
				})
			} catch {
				mods.push({ file })
				cache.deleteString(`mods_${hash}`)
			}
		} else {
			try {
				const mod = await api.modrinth.projectByHash(hash)

				cache.setString(`mods_${hash}`, JSON.stringify(mod))

				mods.push({
					file,
					infos: mod
				})
			} catch {
				mods.push({ file })
			}
		}
	}

	process.stdout.write('\n')
	return mods
}