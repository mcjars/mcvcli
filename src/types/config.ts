import { z } from "zod"

export const configVersions = z.union([
	z.literal(1),
	z.literal(2),
	z.literal(3),
	z.literal(4)
])

export const configSchema = z.object({
	configVersion: configVersions,
	__README: z.literal('This file is used to store the configuration for the mcvcli tool. Do not modify this file unless you know what you are doing.'),
	jarFile: z.string(),
	javaVersion: z.number(),
	profileName: z.string(),
	modpackSlug: z.string().nullable(),
	modpackVersion: z.string().nullable(),
	extraFlags: z.string().array(),
	extraArgs: z.string().array(),
	ramMB: z.number()
})