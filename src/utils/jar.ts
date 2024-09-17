import { filesystem, time } from "@rjweb/utils"
import { existsSync } from "fs"
import { readdir } from "fs/promises"
import { SupportedProject, fetchOptions } from "src/api"
import { Cache } from "src/utils/cache"

type APIResponse = {
  success: false
  errors: string[]
} | {
  success: true
  build: {
    buildNumber: number
    type: Uppercase<SupportedProject>
    versionId: string | null
    projectVersionId: string | null
  }
}

export default async function jarVersion(jar: string, cache: Cache): Promise<{
  type: SupportedProject | 'unknown'
  minecraftVersion: string
  jarVersion: string
}> {
  let hash: string | undefined

  if (existsSync('libraries/net/minecraftforge/forge')) {
    const [ version ] = await readdir('libraries/net/minecraftforge/forge')

    if (version) {
      const files = await readdir(`libraries/net/minecraftforge/forge/${version}`),
        file = files.find((file) => file.endsWith('-server.jar') || file.endsWith('-universal.jar'))

      hash = await filesystem.hash(`libraries/net/minecraftforge/forge/${version}/${file}`, { algorithm: 'sha256' })
    }
  }

  if (existsSync('libraries/net/neoforged/neoforge')) {
    const [ version ] = await readdir('libraries/net/neoforged/neoforge')

    if (version) {
      const files = await readdir(`libraries/net/neoforged/neoforge/${version}`),
        file = files.find((file) => file.endsWith('-server.jar') || file.endsWith('-universal.jar'))

      hash = await filesystem.hash(`libraries/net/neoforged/neoforge/${version}/${file}`, { algorithm: 'sha256' })
    }
  }

  if (!hash && existsSync(jar)) hash = await filesystem.hash(jar, { algorithm: 'sha256' })

  if (!hash) return {
    type: 'unknown',
    minecraftVersion: 'unknown',
    jarVersion: 'unknown'
  }

  const cached = cache.get<{
    type: SupportedProject
    minecraftVersion: string
    jarVersion: string
  }>(`jar_${hash}`)

  if (cached) return cached

  try {
    const build = await fetch('https://versions.mcjars.app/api/v2/build', {
      ...fetchOptions,
      method: 'POST',
      body: JSON.stringify({
        hash: {
          sha256: hash
        }
      }), headers: {
        ...fetchOptions.headers,
        'Content-Type': 'application/json'
      }
    }).then((res) => res.json()) as APIResponse

    if (!build.success) {
      return {
        type: 'unknown',
        minecraftVersion: 'unknown',
        jarVersion: 'unknown'
      }
    }

    cache.set(`jar_${hash}`, {
      type: build.build.type.toLowerCase() as SupportedProject,
      minecraftVersion: build.build.versionId ?? build.build.projectVersionId ?? 'unknown',
      jarVersion: build.build.buildNumber === 1 ? build.build.projectVersionId ?? build.build.buildNumber.toString() : build.build.buildNumber.toString()
    }, time(12).h())

    return {
      type: build.build.type.toLowerCase() as SupportedProject,
      minecraftVersion: build.build.versionId ?? build.build.projectVersionId ?? 'unknown',
      jarVersion: build.build.buildNumber === 1 ? build.build.projectVersionId ?? build.build.buildNumber.toString() : build.build.buildNumber.toString()
    }
  } catch {
    return {
      type: 'unknown',
      minecraftVersion: 'unknown',
      jarVersion: 'unknown'
    }
  }
}