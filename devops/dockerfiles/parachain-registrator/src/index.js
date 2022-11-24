
import { Keyring } from '@polkadot/keyring';
import { ApiPromise, WsProvider } from '@polkadot/api';

import BN from "bn.js";
import fs from "fs";

const state_file = process.env.STATE_FILE ? process.env.STATE_FILE : "/code/genesis-state";
const wasm_file = process.env.WASM_FILE ? process.env.WASM_FILE : "/code/genesis-wasm";
const address = process.env.COLLATOR_ADDR ? process.env.COLLATOR_ADDR : "ws://10.0.0.2:9944";
const paraId = process.env.PARA_ID ? new BN(process.env.PARA_ID) : new BN("2110");

function range(size, startAt = 0) {
	return [...Array(size).keys()].map(i => i + startAt);
}


async function wait_for_new_block(api) {
	let wait = new Promise(async (resolve, _) => {
		let counter = 0;
		const unsub_new_heads = await api.derive.chain.subscribeNewHeads(async (header) => {
			if (counter++ > 0) {
				console.info(`new block produced #${header.number}`);
				unsub_new_heads()
				resolve()
			}
		});
	})

	await wait;
}

async function getNextFreeParaId(api) {
	return (await api.query.registrar.nextFreeParaId()).toBn();
}

async function blockUntilFinalized(tx, account, options, verbose = false) {
	return new Promise(async (resolve, reject) => {
		const unsub = await tx.signAndSend(account, options, async (result) => {
			if (result.status.isFinalized) {
				unsub();
				resolve(result.status.asFinalized.toString())
			} else if (result.status.isInvalid) {
				unsub();
				reject("tx was not executed")
			}
		})
	})
}

async function forceLease(api, sudo, paraId, blockNr) {
	const lease_periods = (await api.query.slots.leases(paraId)).toHuman();

	if (lease_periods.length > 0) {
		console.info(`#${blockNr} Parachain ${paraId}: active leases period found`);
		return Promise.resolve();
	}


	let period = api.consts.slots.leasePeriod.toBn();
	let offset = api.consts.slots.leaseOffset.toBn();


	// Note that blocks before `LeaseOffset` do not count as any lease period.
	let leasePeriod = blockNr.sub(offset).div(period);

	console.info(`#${blockNr} Parachain ${paraId}: no leases found - calling slots::forceLease`);

	return api.tx.sudo.sudo(
		api.tx.slots.forceLease(paraId, sudo.address, 666, leasePeriod, 999)
	).signAndSend(sudo);

}


async function main() {

	const wsProvider = new WsProvider(address);
	const api = await ApiPromise.create({
		provider: wsProvider,
	});

	await api.isReady;
	await wait_for_new_block(api);

	let keyring = new Keyring({ type: "sr25519" });
	const alice = keyring.addFromUri('//Alice');

	let aliceNonce = (await api.rpc.system.accountNextIndex(alice.address)).toBn();

	let nextId = await getNextFreeParaId(api);

	if (nextId.lt(paraId)) {
		console.info(`reserving parachain slots from ${nextId} to ${paraId}`);
		let slotsCount = paraId.sub(nextId);
		let nonces = range(slotsCount.toNumber(), aliceNonce.toNumber());
		aliceNonce.iaddn(nonces.length);
		let slotsReserveTxs = nonces.map((nonce) => blockUntilFinalized(api.tx.registrar.reserve(), alice, { nonce: nonce }));
		await Promise.all(slotsReserveTxs);

		console.info(`Parachain ${paraId} - preparing initialization tx`);
		const genesis = fs.readFileSync(state_file).toString();
		const wasm = fs.readFileSync(wasm_file).toString();
		const scheduleParaInit = api.tx.parasSudoWrapper.sudoScheduleParaInitialize(
			paraId,
			{
				genesisHead: genesis,
				validationCode: wasm,
				parachain: true,
			}
		);

		console.info(`Parachain ${paraId} - sending initialization tx`);
		await blockUntilFinalized(
			api.tx.sudo.sudo(scheduleParaInit),
			alice,
			{ nonce: aliceNonce },
		);
		console.info(`Parachain ${paraId} - initialization completed`);
	}

	let init_call = true;
	return new Promise(async (_) => {
		const unsub = await api.derive.chain.subscribeNewHeads(async (header) => {
			if (init_call || header.number.toBn().modn(10) == new BN(0)) {
				init_call = false;
				await forceLease(api, alice, paraId, header.number.toBn());
			}
		})

	});
};


main().catch(console.error).finally(() => process.exit());
