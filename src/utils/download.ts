import { trackResponseProgress,  } from "fetch-api-progress"
import { number } from "@rjweb/utils"
import chalk from "chalk"
import fs from "fs"
import bytes from "bytes"

export default async function download(display: string, url: string, dest: string, size?: number | null) {
	const request = await fetch(url)

	if (!request.body) throw new Error('no body')

	const tracked = trackResponseProgress(request, (progress) => {
		const percent = number.limit(Math.round(progress.loaded / (progress.total ?? size ?? progress.loaded) * 100), 99)

		process.stdout.write(`\rdownloading ${chalk.cyan(display)} ${percent}% ${progress.total || size ? `(${bytes(progress.loaded)} / ${bytes(progress.total ?? size ?? 0)})` : ''}      `)
	})

	const file = fs.createWriteStream(dest)

	for await (const chunk of tracked.body!) {
		file.write(chunk)
	}

	file.close()
	process.stdout.write('\n')
}