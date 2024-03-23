import { trackResponseProgress,  } from "fetch-api-progress"
import { number } from "@rjweb/utils"
import chalk from "chalk"
import fs from "fs"

export default async function download(display: string, url: string, dest: string) {
	const request = await fetch(url)

	const tracked = trackResponseProgress(request, (progress) => {
		const percent = number.limit(Math.round(progress.loaded / (progress.total ?? progress.loaded) * 100), 99)

		process.stdout.write(`\rdownloading ${chalk.cyan(display)} ${percent}%`)
	})

	const data = await tracked.blob()
	await fs.promises.writeFile(dest, Buffer.from(await data.arrayBuffer()))
	process.stdout.write('\n')
}