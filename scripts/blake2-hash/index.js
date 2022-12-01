#!/usr/bin/env node

import _yargs from 'yargs';
import { hideBin } from 'yargs/helpers';
const yargs = _yargs(hideBin(process.argv));
import { blake2AsHex } from '@polkadot/util-crypto';
import fs from "fs";


const main = async () => {

	const cli = yargs
		.option("i", {
			alias: "input",
			type: "string",
			demandOption: "The input is required.",
			type: "string",
		})

	const { input } = cli.argv;
	const wasm = fs.readFileSync(input);
	const hash = blake2AsHex(wasm)
	console.log(hash)
};

main().then(() => process.exit(0));
