import bytes from "bytes"
import chalk from "chalk"
import getCache from "src/utils/cache"

export type Args = {}

export default async function cacheClear(args: Args) {
	const cache = getCache(),
		size = cache.size()

	if (!size) {
		console.log('cache is already empty!')
		process.exit(1)
	}

	for (const key of cache.keys()) {
		cache.delete(key)
	}

	console.log('cleared cache!', chalk.cyan(`-${bytes(size)}`))
}