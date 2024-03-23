import { z } from "zod"

export const configSchema = z.object({
	__README: z.literal('This file is used to store the configuration for the mccli tool. Do not modify this file unless you know what you are doing.'),
	jarFile: z.string(),
	profileName: z.string(),
	ramMB: z.number()
})