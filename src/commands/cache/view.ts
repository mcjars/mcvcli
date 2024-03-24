import bytes from "bytes"
import chalk from "chalk"
import getCache from "src/utils/cache"

export type Args = {}

export default async function cacheView(args: Args) {
	const cache = getCache()

	console.log('cache contents:')
	console.log('  location:', chalk.cyan(cache.directory))
	console.log('  keys:', chalk.cyan(cache.keys().length))
	console.log('  size:', chalk.cyan(bytes(cache.size())))
}