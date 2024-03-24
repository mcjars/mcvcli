import fs from "fs"
import path from "path"
import os from "os"

export class Cache {
	constructor(public directory: string) {}

	public getString(key: string, set?: string): string | null {
		const location = path.join(this.directory, 'strings', `${key}.txt`)

		if (!fs.existsSync(location) && !set) {
			return null
		} else if (!set) {
			return fs.readFileSync(location, 'utf-8')
		}

		if (set) {
			fs.mkdirSync(path.join(this.directory, 'strings'), { recursive: true })
			fs.writeFileSync(location, set)
		}

		return set
	}

	public setString(key: string, value: string) {
		this.getString(key, value)
	}

	public deleteString(key: string) {
		const location = path.join(this.directory, 'strings', `${key}.txt`)

		if (fs.existsSync(location)) {
			fs.rmSync(location)
		}
	}
}

export default function getCache(directory: string = path.join(os.userInfo().homedir, '.mccli', 'cache')) {
	if (!fs.existsSync(directory)) {
		fs.mkdirSync(directory, { recursive: true })
	}

	return new Cache(directory)
}