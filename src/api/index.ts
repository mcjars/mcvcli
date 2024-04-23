import AdmZip from "adm-zip"
import path from "path"
import fs from "fs"
import doDownload from "src/utils/download"
import { Config } from "src/utils/config"
import { version } from "../../package.json"
import chalk from "chalk"

export const fetchOptions: RequestInit = {
	headers: {
		'User-Agent': `github.com/0x7d8/mccli v${version} (https://rjansen.dev)`
	}
}

export * as modrinth from "src/api/modrinth"

export const supportedProjects = ['paper', 'purpur', 'fabric', 'quilt', 'folia', 'velocity', 'waterfall', 'bungeecord', 'vanilla', 'forge'] as const
export type SupportedProject = typeof supportedProjects[number]

export async function player(identifier: string): Promise<{
	uuid: string
	username: string
	created_at: string
} | null> {
	const res = await fetch(`https://api.ashcon.app/mojang/v2/user/${identifier}`)
	if (res.status === 404) return null

	return res.json() as any
}

export async function latest(project: SupportedProject | 'unknown', mc: string): Promise<{
	latestJar: string
	latestMc: string
}> {
	if (project === 'unknown') {
		return {
			latestJar: 'unknown',
			latestMc: 'unknown'
		}
	}

	const res = await fetch(`https://mc.rjns.dev/api/v1/builds/${project}`, fetchOptions).then((res) => res.json()) as {
		success: true
		versions: Record<string, {
			latest: {
				versionId: string | null
				projectVersionId: string | null
				buildNumber: number
			}
		}>
	}

	return {
		latestJar: res.versions[mc].latest.buildNumber === 1 ? res.versions[mc].latest.projectVersionId ?? res.versions[mc].latest.buildNumber.toString() : res.versions[mc].latest.buildNumber.toString(),
		latestMc: Object.values(res.versions)[0]?.latest.versionId ?? Object.values(res.versions)[0]?.latest.projectVersionId ?? 'unknown'
	}
}

export async function versions(project: SupportedProject): Promise<string[]> {
	const res = await fetch(`https://mc.rjns.dev/api/v1/builds/${project}`, fetchOptions).then((res) => res.json()) as {
		success: true
		versions: Record<string, unknown>
	}

	return Object.keys(res.versions)
}

export async function builds(project: SupportedProject, mc: string): Promise<{
	jarVersion: string
	mcVersion: string

	download: {
		jar: string | null
		zip: string | null
	}
}[]> {
	const res = await fetch(`https://mc.rjns.dev/api/v1/builds/${project}/${mc}`, fetchOptions).then((res) => res.json()) as {
		success: true
		builds: {
			buildNumber: number
			versionId: string | null
			projectVersionId: string | null
			jarUrl: string | null
			zipUrl: string | null
		}[]
	}

	return res.builds.map((build) => ({
		jarVersion: build.projectVersionId ?? build.buildNumber.toString(),
		mcVersion: build.versionId ?? 'unknown',
		download: {
			jar: build.jarUrl,
			zip: build.zipUrl
		}
	}))
}

export async function install(download: {
	jar: string | null
	zip: string | null
}, config: Config) {
	if (download.jar && !download.zip) {
		await doDownload('server.jar', download.jar, config.data.jarFile)
	} else if (download.zip) {
		const zipName = download.zip.split('/').pop()?.slice(0, -4)!,
			fileName = `${zipName}.zip`

		await doDownload(fileName, download.zip, path.join(path.dirname(config.data.jarFile), fileName))

		const archive = new AdmZip(path.join(path.dirname(config.data.jarFile), fileName))
		archive.extractAllTo(path.dirname(config.data.jarFile), true)

		await fs.promises.unlink(path.join(path.dirname(config.data.jarFile), fileName))

		config.data.jarFile = zipName
		config.write()

		if (download.jar) {
			if (fs.existsSync(path.join(path.dirname(config.data.jarFile), '.mcvapi.jarUrl.txt'))) {
				const fileName = await fs.promises.readFile(path.join(path.dirname(config.data.jarFile), '.mcvapi.jarUrl.txt'), 'utf-8')
				await fs.promises.unlink(path.join(path.dirname(config.data.jarFile), '.mcvapi.jarUrl.txt'))

				await doDownload('server.jar', download.jar, path.join(path.dirname(config.data.jarFile), fileName))
			} else {
				await doDownload('server.jar', download.jar, path.join(path.dirname(config.data.jarFile), 'server.jar'))
			}
		}
	}
}

export async function searchModpacks(query: string) {
	const res = await fetch(`https://api.modrinth.com/v2/search?query=${encodeURIComponent(query)}&facets=[["project_type:modpack"],["server_side:required","server_side:optional"]]`, fetchOptions).then((res) => res.json()) as{
		hits: {
			title: string
			id: string
			slug: string
			versions: string[]
		}[]
	}

	return res.hits.map((hit) => ({
		title: hit.title,
		slug: hit.slug ?? hit.id,
		versions: hit.versions
	}))
}

export async function installModpack(slug: string, oldVersionId: string | null, versionId: string, config: Config) {
	const versions = await modpackVersions(slug),
		version = versions.find((v) => v.id === versionId)!,
		oldVersion = versions.find((v) => v.id === oldVersionId)

	config.data.modpackVersion = version.id

	if (oldVersion) try {
		await doDownload('old_modpack.mrpack', oldVersion.files.find((file) => file.primary)?.url ?? oldVersion.files[0].url, path.join(path.dirname(config.data.jarFile), 'modpack.mrpack'))

		const archive = new AdmZip(path.join(path.dirname(config.data.jarFile), 'modpack.mrpack')),
			index = JSON.parse(archive.readAsText('modrinth.index.json')) as {
				dependencies: Record<string, string>
				files: {
					path: string
				}[]
			}

		for (const file of index.files) {
			await fs.promises.rm(path.join(path.dirname(config.data.jarFile), file.path), { recursive: true })
			console.log('removed', chalk.cyan(file.path), 'from old version')
		}

		for (const file of archive.getEntries()) {
			if (file.entryName.startsWith('overrides/')) {
				const filePath = file.entryName.slice(10)

				await fs.promises.rm(path.join(path.dirname(config.data.jarFile), filePath), { recursive: true })
				console.log('removed', chalk.cyan(filePath), 'from old version')
			}
		}
	} catch {
		console.log('error reading old version! you may have to manually remove the old files.')
	} finally {
		await fs.promises.rm(path.join(path.dirname(config.data.jarFile), 'modpack.mrpack'))
	}

	try {
		await doDownload('modpack.mrpack', version.files.find((file) => file.primary)?.url ?? version.files[0].url, path.join(path.dirname(config.data.jarFile), 'modpack.mrpack'))

		const archive = new AdmZip(path.join(path.dirname(config.data.jarFile), 'modpack.mrpack')),
			index = JSON.parse(archive.readAsText('modrinth.index.json')) as {
				dependencies: Record<string, string>
				files: {
					downloads: string[]
					path: string
					env?: {
						server: 'required' | 'optional' | 'unsupported'
					}
				}[]
			}

		const versionBuilds = await builds(version.loaders[0] as SupportedProject, index.dependencies.minecraft ?? version.game_versions[0])

		const loaderVersion = index.dependencies[`${version.loaders[0]}-loader`] ?? index.dependencies[`${version.loaders[0]}-api`],
			jar = versionBuilds.find((build) => build.jarVersion === loaderVersion) ?? versionBuilds[0]

		await install(jar.download, config)

		for (const file of index.files) {
			if (file.env?.server !== 'unsupported') {
				await fs.promises.mkdir(path.dirname(path.join(path.dirname(config.data.jarFile), file.path)), { recursive: true })
				await doDownload(file.path, file.downloads[0], path.join(path.dirname(config.data.jarFile), file.path))
			}
		}

		const overrides = archive.getEntry('overrides')
		if (overrides?.isDirectory) {
			archive.extractEntryTo(overrides, path.dirname(config.data.jarFile), false, true)
		}
	} catch {
		console.log('unsupported loader or game version!')
		process.exit(1)
	} finally {
		await fs.promises.rm(path.join(path.dirname(config.data.jarFile), 'modpack.mrpack'))
	}
}

export async function latestModpack(slug: string): Promise<{
	latestVersion: string
}> {
	const res = await fetch(`https://api.modrinth.com/v2/project/${slug}/version`, fetchOptions).then((res) => res.json()) as {
		version_number: string
	}[]

	return {
		latestVersion: res[0].version_number
	}
}

export async function modpackVersions(slug: string) {
	const res = await fetch(`https://api.modrinth.com/v2/project/${slug}/version`, fetchOptions).then((res) => res.json()) as {
		id: string
		name: string
		game_versions: string[]
		version_number: string
		loaders: string[]
		files: {
			url: string
			primary: boolean
		}[]
	}[]

	return res.map((version) => ({
		id: version.id,
		title: version.name ?? version.version_number,
		game_versions: version.game_versions,
		version_number: version.version_number,
		loaders: version.loaders,
		files: version.files
	}))
}

export async function modpackInfos(slug: string) {
	const res = await fetch(`https://api.modrinth.com/v2/project/${slug}`, fetchOptions).then((res) => res.json()) as {
		title: string
		server_side: 'required' | 'optional' | 'unsupported'
		license: {
			id: string
		}
	}

	return res
}