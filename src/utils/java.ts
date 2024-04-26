import fs from "fs"
import path from "path"
import os from "os"
import * as api from "src/api"
import download from "src/utils/download"
import AdmZip from "adm-zip"
import * as tar from "tar"

const location = path.join(os.userInfo().homedir, '.mccli', 'java')

export function installed(): number[] {
	const out: number[] = []

	if (!fs.existsSync(location)) {
		fs.mkdirSync(location, { recursive: true })
	}

	for (const dir of fs.readdirSync(location)) {
		const int = parseInt(dir)

		if (!isNaN(int) && fs.existsSync(path.join(location, dir, 'bin', 'java'))) {
			out.push(int)
		}
	}

	return out
}

export async function binary(version: number): Promise<string> {
	const current = installed()

	if (current.includes(version)) {
		return path.join(location, version.toString(), 'bin', 'java')
	}

	if (!fs.existsSync(path.join(location, version.toString()))) {
		fs.mkdirSync(path.join(location, version.toString()), { recursive: true })
	}

	const [ name, url ] = await api.adoptium.packagedUrl(version),
		dest = path.join(location, version.toString(), 'java.zip')

	await download(name, url, dest)
	if (name.endsWith('.zip')) {
		new AdmZip(dest).extractAllTo(path.join(location, version.toString()), true)
		await fs.promises.rm(dest)

		const folder = fs.readdirSync(path.join(location, version.toString()))[0]

		fs.readdirSync(path.join(location, version.toString(), folder)).forEach((file) => {
			fs.renameSync(path.join(location, version.toString(), folder, file), path.join(location, version.toString(), file))
		})
	} else {
		await tar.extract({ file: dest, cwd: path.join(location, version.toString()) })
		await fs.promises.rm(dest)

		const folder = fs.readdirSync(path.join(location, version.toString()))[0]

		fs.readdirSync(path.join(location, version.toString(), folder)).forEach((file) => {
			fs.renameSync(path.join(location, version.toString(), folder, file), path.join(location, version.toString(), file))
		})
	}

	return path.join(location, version.toString(), 'bin', 'java')
}