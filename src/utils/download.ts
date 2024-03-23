import { number } from "@rjweb/utils"
import chalk from "chalk"
import fs from "fs"
import bytes from "bytes"

export default async function download(display: string, url: string, dest: string, overrideSize?: number | null) {
	const request = await fetch(url)

	if (!request.body) throw new Error('no body')

	const size = overrideSize ?? request.headers.has('content-length') ? parseInt(request.headers.get('content-length') ?? '0') : null,
		file = fs.createWriteStream(dest)

	let progress = 0
	for await (const chunk of request.body) {
		progress += chunk.length
		const percent = number.limit(Math.round(progress / (size ?? chunk.length) * 100), 99)

		process.stdout.write(`\rdownloading ${chalk.cyan(display)} ${percent}% ${size ? `(${bytes(progress)} / ${bytes(size ?? 0)})` : ''}      `)

		await new Promise<void>((resolve) => {
			file.write(chunk, () => resolve())
		})
	}

	file.close()
	process.stdout.write('\n')
}