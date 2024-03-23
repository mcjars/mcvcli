import fs from "fs"
import cp from "child_process"
import path from "path"
import getConfig from "src/utils/config"

export type Args = {}

export default function start(args: Args) {
	const config = getConfig()

	console.log('starting server...')
	console.log(`java -Xmx${config.data.ramMB}M -jar ${config.data.jarFile} nogui`)

	fs.writeFileSync('eula.txt', 'eula=true')

	const child = cp.spawn('java', [`-Xmx${config.data.ramMB}M`, '-jar', config.data.jarFile, 'nogui'], {
		cwd: path.dirname(config.data.jarFile)
	})

	child.stdout.pipe(process.stdout)
	child.stderr.pipe(process.stderr)
	process.stdin.pipe(child.stdin)

	let isNuke = false
	child.on('exit', (code) => {
		if (!isNuke) console.log('server stopped with code', code)
	})

	process.on('SIGINT', () => {
		child.stdout.unpipe(process.stdout)
		child.kill('SIGINT')
		console.log('server killed. please use exit if possible.')
		isNuke = true
	})
}