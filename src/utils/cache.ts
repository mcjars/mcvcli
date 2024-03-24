import fs from "fs"
import path from "path"
import os from "os"

type CacheFile = {
	data: any
	expires: number | null
}

export class Cache {
	constructor(public directory: string) {}

	public get<Data extends object = object>(key: string): Data | null {
		const location = path.join(this.directory, 'basic', `${key}.json`)

		if (!fs.existsSync(location)) {
			return null
		} else {
			const content = JSON.parse(fs.readFileSync(location, 'utf-8')) as CacheFile

			if (content.expires && content.expires < Date.now()) {
				fs.rmSync(location)
				return null
			}

			return content.data
		}
	}

	public set(key: string, value: object, expires?: number) {
		const location = path.join(this.directory, 'basic', `${key}.json`)

		fs.mkdirSync(path.dirname(location), { recursive: true })
		fs.writeFileSync(location, JSON.stringify({ data: value, expires: expires && Date.now() + expires }))
	}

	public delete(key: string) {
		const location = path.join(this.directory, 'basic', `${key}.json`)

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