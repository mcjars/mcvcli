import fs from "fs"
import cp from "child_process"
import path from "path"
import getConfig from "src/utils/config"
import enquirer from "enquirer"

export type Args = {}

export default async function start(args: Args) {
	const config = getConfig()

	const eula = await fs.promises.readFile('eula.txt', 'utf8').catch(() => '').then((eula) => eula.includes('eula=true'))

	if (eula) {
		console.log('eula already accepted!')
	} else {
		const { accept } = await enquirer.prompt<{
			accept: boolean
		}>({
			type: 'confirm',
			message: 'Do you accept the Minecraft EULA? (https://www.minecraft.net/eula)',
			name: 'accept'
		})

		if (!accept) {
			console.log('eula not accepted, server will not start!')
			process.exit(1)
		} else {
			await fs.promises.writeFile('eula.txt', 'eula=true')
		}
	}

	console.log('starting server...')
	console.log(`java -Xmx${config.data.ramMB}M -jar ${config.data.jarFile} nogui`)

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