import { filesystem, time } from "@rjweb/utils"
import { existsSync } from "fs"
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
  if (!existsSync(jar)) return {
    type: 'unknown',
    minecraftVersion: 'unknown',
    jarVersion: 'unknown'
  }

  const hash = await filesystem.hash(jar, { algorithm: 'sha256' })

  const cached = cache.get<{
    type: SupportedProject
    minecraftVersion: string
    jarVersion: string
  }>(`jar_${hash}`)

  if (cached) return cached

  try {
    const build = await fetch(`https://mc.rjns.dev/api/v1/build/${hash}`, fetchOptions).then((res) => res.json()) as APIResponse

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