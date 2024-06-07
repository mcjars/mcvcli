import os from "os"

export async function versions(): Promise<number[]> {
	const res = await fetch('https://api.adoptium.net/v3/info/available_releases')
		.then((res) => res.json()) as {
			available_releases: number[]
		}

	return res.available_releases
}

export async function packagedUrl(version: number): Promise<[name: string, url: string]> {
	const query = new URLSearchParams({
		os: process.platform === 'win32' ? 'windows' : process.platform === 'darwin' ? 'mac' : 'linux',
		architecture: os.arch()
	})

	const res = await fetch(`https://api.adoptium.net/v3/assets/latest/${version}/hotspot?${query.toString()}`)
		.then((res) => res.json()) as {
			binary: {
				image_type: string
				package: {
					link: string
					name: string
				}
			}
		}[]

	const packaged = res.find((asset) => asset.binary.image_type === 'jdk' && (asset.binary.package.name.endsWith('.zip') || asset.binary.package.name.endsWith('.tar.gz')))
	if (!packaged) throw new Error(`No JDK found for ${version}`)

	return [packaged.binary.package.name, packaged.binary.package.link]
}