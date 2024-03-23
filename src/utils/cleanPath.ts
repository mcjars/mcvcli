import path from "path/posix"

export default function cleanPath(input: string): string {
	return path.normalize(input.replace(/\.|\/||\\/g, '').slice(0, 50))
}