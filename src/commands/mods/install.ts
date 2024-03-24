import fs from "fs"
import path from "path"
import getCache from "src/utils/cache"
import getConfig from "src/utils/config"
import { getMods } from "src/utils/mods"
import enquirer from "enquirer"
import chalk from "chalk"
import * as api from "src/api"
import getJarVersion from "src/utils/jar"
import download from "src/utils/download"
import { filesystem, time } from "@rjweb/utils"

export type Args = {}

export default async function modsInstall(args: Args) {
	const config = getConfig(),
		cache = getCache(),
		jar = await getJarVersion(config.data.jarFile, cache),
		mods = await getMods(path.join(path.dirname(config.data.jarFile), 'mods'), jar, cache)

	if (!fs.existsSync(path.join(path.dirname(config.data.jarFile), 'mods'))) {
		fs.mkdirSync(path.join(path.dirname(config.data.jarFile), 'mods'))
	}

	if (jar.type === 'unknown') {
		console.log('unknown jar type!')
		process.exit(1)
	}

	if (jar.minecraftVersion === 'unknown') {
		console.log('unknown minecraft version!')
		process.exit(1)
	}

	const facets = `[["project_type:mod"],["server_side:required","server_side:optional"],["categories:${jar.type}"],["versions:${jar.minecraftVersion}"]]`,
		initialPacks = await api.modrinth.search('', facets)

	const { modSlug } = await enquirer.prompt<{
		modSlug: string
	}>({
		type: 'autocomplete',
		message: 'Modrinth Mod',
		name: 'modSlug',
		choices: initialPacks.map((pack) => ({ message: pack.title, value: pack.id })) as any,
		// @ts-ignore
		async suggest(input: string) {
			const mods = await api.modrinth.search(input, facets)
			return mods.map((mod) => ({ message: mod.title, value: mod.id }))
		}
	})

	if (mods.find((mod) => 'infos' in mod && mod.infos.project.id === modSlug)) {
		console.log('mod already installed!')
		process.exit(1)
	}

	const [ data, versions ] = await Promise.all([
		api.modrinth.project(modSlug),
		api.modrinth.versions(modSlug)
	])

	if (!data) {
		console.log('mod not found!')
		process.exit(1)
	}

	console.log('mod found:')
	console.log('  title:', chalk.cyan(data.title))
	console.log('  license:', chalk.cyan(data.license))

	const latest = api.modrinth.latest(versions, jar)

	if (!latest) {
		console.log('no version found!')
		process.exit(1)
	}

	const file = latest.files.find((file) => file.primary) ?? latest.files[0]

	await download(`mods/${file.filename}`, file.url, path.join(path.dirname(config.data.jarFile), 'mods', file.filename))
	const hash = await filesystem.hash(path.join(path.dirname(config.data.jarFile), 'mods', file.filename), { algorithm: 'sha1' })

	cache.set(`mods_${hash}`, {
		id: latest.id,
		title: latest.title ?? latest.version_number,
		version: latest.version_number,
		project: data,
		versions
	}, time(30).m())

	for (const dependency of latest.dependencies) {
		const dep = await api.modrinth.project(dependency.project_id)

		if (!dep) {
			console.log('dependency', chalk.cyan(dependency.project_id), 'not found!')
			continue
		}

		if (mods.find((mod) => 'infos' in mod && mod.infos.project.id === dependency.project_id)) {
			console.log('dependency', chalk.cyan(dep.title), 'already installed!')
			continue
		}

		if (dependency.version_id) {
			const depVersions = await api.modrinth.versions(dependency.project_id),
				depVersion = depVersions.find((v) => v.id === dependency.version_id)

			if (!depVersion) {
				console.log('dependency version', chalk.cyan(dependency.version_id), 'not found!')
				continue
			}

			const depFile = depVersion.files.find((file) => file.primary) ?? depVersion.files[0]

			await download(`mods/${depFile.filename}`, depFile.url, path.join(path.dirname(config.data.jarFile), 'mods', depFile.filename))
			const depHash = await filesystem.hash(path.join(path.dirname(config.data.jarFile), 'mods', depFile.filename), { algorithm: 'sha1' })

			cache.set(`mods_${depHash}`, {
				id: depVersion.id,
				title: depVersion.title ?? depVersion.version_number,
				version: depVersion.version_number,
				project: dep,
				versions: depVersions
			}, time(30).m())
		} else {
			const depLatest = api.modrinth.latest(await api.modrinth.versions(dependency.project_id), jar)

			if (!depLatest) {
				console.log('dependency latest version not found!')
				continue
			}

			const depFile = depLatest.files.find((file) => file.primary) ?? depLatest.files[0]

			await download(`mods/${depFile.filename}`, depFile.url, path.join(path.dirname(config.data.jarFile), 'mods', depFile.filename))
		}
	}

	console.log('mod installed!')
}