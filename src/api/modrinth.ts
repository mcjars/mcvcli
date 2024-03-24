import { fetchOptions } from "src/api"

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

	const project = await mod(res.project_id)

	return {
		id: res.id,
		title: res.name ?? res.version_number,
		version: res.version_number,
		latest: project.versions[0] === res.id,
		project
	}
}

export async function mod(slug: string) {
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

	return {
		id: res.id,
		slug: res.slug ?? res.id,
		title: res.title,
		serverSide: res.server_side,
		versions: res.versions,
		license: res.license.id
	}
}