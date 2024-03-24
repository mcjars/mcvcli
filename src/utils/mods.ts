import { filesystem, number, time } from "@rjweb/utils"
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
	latest: Awaited<ReturnType<typeof api['modrinth']['latest']>>
}

export async function getMods(directory: string, jar: {
	type: api.SupportedProject | 'unknown'
  minecraftVersion: string
  jarVersion: string
}, cache: Cache): Promise<Mod[]> {
	if (jar.minecraftVersion === 'unknown' || jar.type === 'unknown') return []

	const files = await fs.promises.readdir(directory).then((files) => files.filter((file) => file.endsWith('.jar')))

	let progress = 0

	const mods: Mod[] = []
	for (const file of files) {
		progress++

		readline.moveCursor(process.stdout, 0, -1)
		process.stdout.write(`\rchecking mod ${chalk.cyan(file)}...                  \n${number.round((progress / files.length) * 100, 2)}% (${progress} / ${files.length})   `)

		const hash = await filesystem.hash(path.join(directory, file), { algorithm: 'sha1' })

		const mod = cache.get<Awaited<ReturnType<typeof api['modrinth']['projectByHash']>>>(`mods_${hash}`)
		if (mod) {
			mods.push({
				file,
				infos: mod,
				latest: api.modrinth.latest(mod, jar)
			})
		} else {
			try {
				const mod = await api.modrinth.projectByHash(hash)

				cache.set(`mods_${hash}`, mod, time(30).m())

				mods.push({
					file,
					infos: mod,
					latest: api.modrinth.latest(mod, jar)
				})
			} catch {
				mods.push({ file })
			}
		}
	}

	process.stdout.write('\n')
	return mods
}