import AdmZip from "adm-zip"
import path from "path"
import fs from "fs"
import doDownload, { Download } from "src/utils/download"
import { Config } from "src/utils/config"
import { version } from "../../package.json"
import chalk from "chalk"

export const fetchOptions: RequestInit = {
	headers: {
		'User-Agent': `github.com/0x7d8/mcvcli v${version} (https://rjansen.dev)`
	}
}

export * as modrinth from "src/api/modrinth"
export * as adoptium from "src/api/adoptium"

export type InstallationStep = {
	type: 'download'

	file: string
	url: string
	size: number
} | {
	type: 'unzip'

	file: string
	location: string
} | {
	type: 'remove'

	location: string
}

export const supportedProjects = ['paper', 'pufferfish', 'purpur', 'fabric', 'quilt', 'folia', 'velocity', 'waterfall', 'bungeecord', 'sponge', 'leaves', 'vanilla', 'forge', 'neoforge', 'mohist', 'arclight'] as const
export type SupportedProject = typeof supportedProjects[number]

function formatUUID(uuid: string) {
	return `${uuid.slice(0, 8)}-${uuid.slice(8, 12)}-${uuid.slice(12, 16)}-${uuid.slice(16, 20)}-${uuid.slice(20)}`
}

export async function player(identifier: string): Promise<{
	uuid: string
	username: string
} | null> {
	if (identifier.length === 36) {
		const res = await fetch(`https://sessionserver.mojang.com/session/minecraft/profile/${identifier}`, fetchOptions)
		if (!res.ok) return null

		const data = await res.json() as { id: string, name: string }
		return {
			uuid: formatUUID(data.id),
			username: data.name
		}
	} else {
		const res = await fetch(`https://api.mojang.com/users/profiles/minecraft/${identifier}`, fetchOptions)
		if (!res.ok) return null

		const data = await res.json() as { id: string, name: string }
		return {
			uuid: formatUUID(data.id),
			username: data.name
		}
	}
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

	const res = await fetch(`https://versions.mcjars.app/api/v2/builds/${project}`, fetchOptions).then((res) => res.json()) as {
		success: true
		builds: Record<string, {
			latest: {
				versionId: string | null
				projectVersionId: string | null
				buildNumber: number
			}
		}>
	}

	return {
		latestJar: res.builds[mc].latest.buildNumber === 1 ? res.builds[mc].latest.projectVersionId ?? res.builds[mc].latest.buildNumber.toString() : res.builds[mc].latest.buildNumber.toString(),
		latestMc: Object.values(res.builds).at(-1)?.latest.versionId ?? Object.values(res.builds).at(-1)?.latest.projectVersionId ?? 'unknown'
	}
}

export async function versions(project: SupportedProject): Promise<{ version: string, java: number }[]> {
	const res = await fetch(`https://versions.mcjars.app/api/v2/builds/${project}`, fetchOptions).then((res) => res.json()) as {
		success: true
		builds: Record<string, {
			java?: number
		}>
	}

	return Object.entries(res.builds).map(([version, build]) => ({ version, java: build.java ?? 21 }))
}

export async function builds(project: SupportedProject, mc: string): Promise<{
	id: number
	jarVersion: string
	mcVersion: string

	download: InstallationStep[][]
}[]> {
	const res = await fetch(`https://versions.mcjars.app/api/v2/builds/${project}/${mc}`, fetchOptions).then((res) => res.json()) as {
		success: true
		builds: {
			id: number
			buildNumber: number
			versionId: string | null
			projectVersionId: string | null
			installation: InstallationStep[][]
		}[]
	}

	return res.builds.map((build) => ({
		id: build.id,
		jarVersion: build.projectVersionId ?? build.buildNumber.toString(),
		mcVersion: build.versionId ?? 'unknown',
		download: build.installation
	}))
}

export async function install(steps: InstallationStep[][], config: Config) {
	for (const segment of steps) {
		const promises: any[] = [],
			downloads: Download[] = []

		for (const step of segment) {
			if (step.type === 'download') {
				await fs.promises.mkdir(path.dirname(path.join(path.dirname(config.data.jarFile), step.file)), { recursive: true })

				downloads.push({
					display: step.file,
					url: step.url,
					dest: path.join(path.dirname(config.data.jarFile), step.file)
				})
			} else if (step.type === 'unzip') {
				const archive = new AdmZip(path.join(path.dirname(config.data.jarFile), step.file))

				archive.extractAllTo(path.join(path.dirname(config.data.jarFile), step.location), true)
			} else if (step.type === 'remove') {
				promises.push(fs.promises.rm(path.join(path.dirname(config.data.jarFile), step.location), { recursive: true }))
			}
		}

		await doDownload(downloads)

		const ranSegments = segment.filter((step) => step.type !== 'download').map((step) => step.type)
		if (ranSegments.length === 0) continue

		const start = performance.now()
		console.log('running', ranSegments.join(', '), '...')
		await Promise.allSettled(promises)
		console.log('running', ranSegments.join(', '), '...', chalk.gray(`(${(performance.now() - start).toFixed(2)}ms)`), chalk.green('done'))
	}

	config.data.jarFile = 'server.jar'
	config.write()
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
		await doDownload([
			{
				display: 'old_modpack.mrpack',
				url: oldVersion.files.find((file) => file.primary)?.url ?? oldVersion.files[0].url,
				dest: path.join(path.dirname(config.data.jarFile), 'modpack.mrpack')
			}
		])

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
		await doDownload([
			{
				display: 'old_modpack.mrpack',
				url: version.files.find((file) => file.primary)?.url ?? version.files[0].url,
				dest: path.join(path.dirname(config.data.jarFile), 'modpack.mrpack')
			}
		])

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

		const chunkSize = 5
		for (let i = 0; i < index.files.length; i += chunkSize) {
			const chunk = index.files.slice(i, i + chunkSize)

			for (const c of chunk) {
				await fs.promises.mkdir(path.dirname(path.join(path.dirname(config.data.jarFile), c.path)),{ recursive: true })
			}

			await doDownload(chunk.map((c) => ({
				display: c.path,
				url: c.downloads[0],
				dest: path.join(path.dirname(config.data.jarFile), c.path)
			})))
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