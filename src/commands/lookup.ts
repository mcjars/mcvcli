import fs from "fs"
import chalk from "chalk"
import * as api from "src/api"
import { parse } from "prismarine-nbt"

export type Args = {
	player: string
}

export default async function lookup(args: Args) {
	console.log('looking up player...')
	const player = await api.player(args.player)

	if (!player) {
		console.log('player not found!')
		process.exit(1)
	}

	const ops: string[] = [],
		opsExist = fs.existsSync('ops.json')
	if (opsExist) {
		try {
			ops.push(...JSON.parse(fs.readFileSync('ops.json', 'utf8')).map((op: { uuid: string }) => op.uuid))
		} catch {}
	}

	if (fs.existsSync('.mccli.json') && fs.existsSync('world') && fs.existsSync('world/playerdata') && fs.existsSync(`world/playerdata/${player.uuid}.dat`)) {
		try {
			const nbt: any = await parse(fs.readFileSync(`world/playerdata/${player.uuid}.dat`))

			console.log('player:')
			console.log('uuid:', chalk.cyan(player.uuid))
			console.log('username:', chalk.cyan(player.username))
			if (opsExist) console.log('operator:', chalk.cyan(ops.includes(player.uuid) ? 'yes' : 'no'))
			console.log('health:', chalk.cyan(`${nbt.parsed.value.Health.value} / ${nbt.parsed.value.Attributes.value.value.find((attr: { Name: { value: string } }) => /health/i.test(attr.Name.value))?.Base?.value ?? 20}`))
			console.log('xp:', chalk.cyan(nbt.parsed.value.XpLevel.value.toString()))
			if (nbt.parsed.value["Spigot.ticksLived"]) console.log('ticks lived:', chalk.cyan(nbt.parsed.value["Spigot.ticksLived"].value.toString()))
			console.log('inventory:')
			console.log('  filled slots:', chalk.cyan(nbt.parsed.value.Inventory.value.value.length.toString()))
			console.log('  total items:', chalk.cyan(nbt.parsed.value.Inventory.value.value.reduce((acc: number, item: any) => acc + item.Count.value, 0).toString()))
			console.log('position:')
			console.log('  world:', chalk.cyan(nbt.parsed.value.Dimension.value))
			console.log('  flying:', chalk.cyan(nbt.parsed.value.abilities.value.flying.value ? 'yes' : 'no'))
			console.log('  x:', chalk.cyan(nbt.parsed.value.Pos.value.value[0].toString()))
			console.log('  y:', chalk.cyan(nbt.parsed.value.Pos.value.value[1].toString()))
			console.log('  z:', chalk.cyan(nbt.parsed.value.Pos.value.value[2].toString()))
		} catch {
			console.log('player:')
			console.log('uuid:', chalk.cyan(player.uuid))
			console.log('username:', chalk.cyan(player.username))
			if (opsExist) console.log('operator:', chalk.cyan(ops.includes(player.uuid) ? 'yes' : 'no'))
			console.log('(failed to get more player data information from nbt files)')
		}
	} else {
		console.log('player:')
		console.log('uuid:', chalk.cyan(player.uuid))
		console.log('username:', chalk.cyan(player.username))
		if (opsExist) console.log('operator:', chalk.cyan(ops.includes(player.uuid) ? 'yes' : 'no'))
		console.log('(navigate to a server directory to view more player data)')
	}
}