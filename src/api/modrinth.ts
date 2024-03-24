import { SupportedProject, fetchOptions } from "src/api"

export async function projectByHash(sha1: string) {
	const res = await fetch('https://api.modrinth.com/v2/version_files', {
		...fetchOptions,
		headers: {
			...fetchOptions.headers,
			'Content-Type': 'application/json'
		}, method: 'POST',
		body: JSON.stringify({
			hashes: [sha1],
			algorithm: 'sha1'
		})
	}).then((res) => res.json()).then((res) => (res as any)[sha1]) as {
		id: string
		project_id: string
		name: string
		version_number: string
	}

	const [ projectData, versionsData ] = await Promise.all([
		project(res.project_id),
		versions(res.project_id)
	])

	return {
		id: res.id,
		title: res.name ?? res.version_number,
		version: res.version_number,
		project: projectData!,
		versions: versionsData
	}
}

export function latest(versions: Awaited<ReturnType<typeof projectByHash>>['versions'], jar: {
	type: SupportedProject | 'unknown'
  minecraftVersion: string
}) {
	return versions.find((version) => version.loaders.includes(jar.type) && version.game_versions.includes(jar.minecraftVersion))
}

export async function project(slug: string) {
	const res = await fetch(`https://api.modrinth.com/v2/project/${slug}`, fetchOptions).then((res) => res.json()) as {
		title: string
		id: string
		slug: string | null
		server_side: 'required' | 'optional' | 'unsupported'
		versions: string[]
		license: {
			id: string
		}
	}

	if (!res) return null

	return {
		id: res.id,
		slug: res.slug ?? res.id,
		title: res.title,
		serverSide: res.server_side,
		versions: res.versions,
		license: res.license.id
	}
}

export async function projects(slugs: string[]) {
	const res = await fetch(`https://api.modrinth.com/v2/projects?ids=${JSON.stringify(slugs)}`, fetchOptions).then((res) => res.json()) as {
		id: string
		title: string
		slug: string
		versions: string[]
		license: string
	}[]

	return res.map((mod) => ({
		id: mod.id,
		slug: mod.slug ?? mod.id,
		title: mod.title,
		versions: mod.versions,
		license: mod.license
	}))
}

export async function versions(slug: string) {
	const res = await fetch(`https://api.modrinth.com/v2/project/${slug}/version`, fetchOptions).then((res) => res.json()) as {
		id: string
		name: string
		game_versions: string[]
		version_number: string
		loaders: string[]
		files: {
			url: string
			filename: string
			primary: boolean
		}[]
		dependencies: {
			version_id: string | null
			project_id: string
		}[]
	}[]

	if (!res) return []

	return res.map((version) => ({
		id: version.id,
		title: version.name ?? version.version_number,
		game_versions: version.game_versions,
		version_number: version.version_number,
		loaders: version.loaders,
		files: version.files,
		dependencies: version.dependencies
	}))
}

export async function search(query: string, facets: string) {
	const res = await fetch(`https://api.modrinth.com/v2/search?query=${encodeURIComponent(query)}&facets=${facets}`, fetchOptions).then((res) => res.json()) as{
		hits: {
			title: string
			project_id: string
			slug: string
			versions: string[]
		}[]
	}

	return res.hits.map((hit) => ({
		id: hit.project_id,
		title: hit.title,
		slug: hit.slug ?? hit.project_id,
		versions: hit.versions
	}))
}