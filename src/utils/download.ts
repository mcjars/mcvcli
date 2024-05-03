import { number } from "@rjweb/utils"
import chalk from "chalk"
import fs from "fs"
import bytes from "bytes"
import { fetchOptions } from "src/api"

export default async function download(display: string, url: string, dest: string, overrideSize?: number | null) {
	const request = await fetch(url, fetchOptions)

	if (!request.body) throw new Error('no body')

	const size = overrideSize ?? request.headers.has('content-length') ? parseInt(request.headers.get('content-length') ?? '0') : null,
		file = fs.createWriteStream(dest)

	let progress = 0, realSpeed = 0
	const startTime = Date.now()

	for await (const chunk of request.body) {
		progress += chunk.length
		realSpeed = progress / ((Date.now() - startTime) / 1000)

		const percent = Math.min(Math.round(progress / (size ?? chunk.length) * 100), 99)

		process.stdout.write(`\rdownloading ${chalk.cyan(display)} ${percent}% ${size ? `(${bytes(progress)} / ${bytes(size ?? 0)})` : ''} ${chalk.gray(`(${bytes(realSpeed)}/s)`)}      `)

		await new Promise<void>((resolve) => {
			file.write(chunk, () => resolve())
		})
	}

	file.close()
	process.stdout.write('\n')
}