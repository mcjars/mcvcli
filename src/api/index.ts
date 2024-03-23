import AdmZip from "adm-zip"
import path from "path"
import fs from "fs"
import doDownload from "src/utils/download"
import { Config } from "src/utils/config"

export const supportedProjects = ['paper', 'purpur', 'fabric', 'quilt', 'folia', 'velocity', 'waterfall', 'bungeecord', 'vanilla'] as const
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

	const res = await fetch(`https://mc.rjns.dev/api/v1/builds/${project}`).then((res) => res.json()) as {
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
		latestMc: Object.values(res.versions).at(-1)?.latest.versionId ?? Object.values(res.versions).at(-1)?.latest.projectVersionId ?? 'unknown'
	}
}

export async function versions(project: SupportedProject): Promise<string[]> {
	const res = await fetch(`https://mc.rjns.dev/api/v1/builds/${project}`).then((res) => res.json()) as {
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
	const res = await fetch(`https://mc.rjns.dev/api/v1/builds/${project}/${mc}`).then((res) => res.json()) as {
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
			await doDownload('server.jar', download.jar, path.join(path.dirname(config.data.jarFile), 'server.jar'))
		}
	}
}