import { filesystem } from "@rjweb/utils"
import { existsSync } from "fs"
import { SupportedProject } from "src/api"

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
    created: string
  }
}

export default async function jarVersion(jar: string): Promise<{
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

  try {
    const build = await fetch(`https://mc.rjns.dev/api/v1/build/${hash}`).then((res) => res.json()) as APIResponse

    if (!build.success) {
      return {
        type: 'unknown',
        minecraftVersion: 'unknown',
        jarVersion: 'unknown'
      }
    }

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