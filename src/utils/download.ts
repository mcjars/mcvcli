import chalk from "chalk"
import fs from "fs"
import bytes from "bytes"
import { fetchOptions } from "src/api"
import cliProgress from "cli-progress"

export type Download = {
	display: string
	url: string
	dest: string
	overrideSize?: number | null
}

export default async function download(downloads: Download[]) {
	const maxWidth = downloads.reduce((acc, { display }) => Math.max(acc, display.length), 0)

	const multibar = new cliProgress.MultiBar({
		clearOnComplete: true,
		hideCursor: true,
		barsize: process.stdout.columns - maxWidth - 10,
		format: `{speed} {display} {bar}`
	}, cliProgress.Presets.shades_classic)

	const bars = downloads.map(({ display, dest, overrideSize }) => {
		const bar = multibar.create(overrideSize ?? 1, 0, {
			display: chalk.cyan(display.padStart(maxWidth)),
			speed: chalk.gray('0B/s'.padEnd(10))
		})

		return { bar, dest, overrideSize }
	})

	const requests = downloads.map(({ url }) => fetch(url, fetchOptions)),
	 	responses = await Promise.all(requests)

	await Promise.all(responses.map(async(response, i) => {
		if (!response.body) throw new Error('no body')

		const size = bars[i].overrideSize ?? response.headers.has('content-length') ? parseInt(response.headers.get('content-length') ?? '0') : null,
			file = fs.createWriteStream(bars[i].dest)

		bars[i].bar.setTotal(size ?? 0)

		let progress = 0, realSpeed = 0
		const startTime = Date.now()

		for await (const chunk of response.body) {
			progress += chunk.length
			realSpeed = progress / ((Date.now() - startTime) / 1000)

			bars[i].bar.update(progress, {
				display: chalk.cyan(downloads[i].display.padStart(maxWidth)),
				speed: chalk.gray(`${bytes(realSpeed)}/s`.padEnd(10))
			})
			
			await new Promise<void>((resolve) => {
				file.write(chunk, () => resolve())
			})
		}

		file.close()
		bars[i].bar.stop()
	}))

	multibar.stop()
}