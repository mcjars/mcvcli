import fs from "fs"
import cp from "child_process"
import path from "path"
import getConfig from "src/utils/config"
import enquirer from "enquirer"
import { binary } from "src/utils/java"
import chalk from "chalk"
import { time } from "@rjweb/utils"

export type Args = {}

export default async function start(args: Args) {
	const config = getConfig()

	const binaryLocation = await binary(config.data.javaVersion),
		eula = await fs.promises.readFile('eula.txt', 'utf8').catch(() => '').then((eula) => eula.includes('eula=true'))

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
	console.log(`${binaryLocation} ${config.data.extraFlags.join(' ')} -Xmx${config.data.ramMB}M -jar ${config.data.jarFile} nogui ${config.data.extraArgs.join(' ')}`)

	const child = cp.spawn(binaryLocation, [...config.data.extraFlags, `-Xmx${config.data.ramMB}M`, '-jar', config.data.jarFile, 'nogui', ...config.data.extraArgs], {
		cwd: path.dirname(config.data.jarFile),
		env: {
			...process.env,
			JAVA_HOME: path.resolve(path.dirname(binaryLocation).concat('/..'))
		}
	})

	child.stdout.pipe(process.stdout)
	child.stderr.pipe(process.stderr)
	process.stdin.pipe(child.stdin)

	let isNuke = false, nukeInterval: NodeJS.Timeout
	child.on('exit', (code) => {
		if (!isNuke) console.log('server stopped with code', code)
		if (nukeInterval) clearTimeout(nukeInterval)
	})

	process.on('SIGINT', () => {
		child.kill('SIGINT')
		console.log('server asked to stop. please use', chalk.cyan('stop'), 'or', chalk.cyan('end'), 'to stop the server.')
		console.log(chalk.yellow('if the server does not stop in 10 seconds, it will be SIGKILLed!'))
		isNuke = true

		if (!nukeInterval) nukeInterval = setTimeout(() => {
			child.kill('SIGKILL')
			console.log(chalk.red('server did not stop in time, SIGKILLed!'))
		}, time(10).s())
	})
}