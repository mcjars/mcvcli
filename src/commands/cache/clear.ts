import chalk from "chalk"
import getCache from "src/utils/cache"

export type Args = {}

export default async function cacheClear(args: Args) {
	const cache = getCache(),
		size = cache.size()

	for (const key of cache.keys()) {
		cache.delete(key)
	}

	console.log('cleared cache!', chalk.cyan(`-${size}`))
}