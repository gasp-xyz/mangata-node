
import { Keyring } from '@polkadot/keyring';
import { ApiPromise, WsProvider } from '@polkadot/api';

import BN from "bn.js";
import fs from "fs";
import { exit } from 'process';

const state_file = process.env.STATE_FILE ? process.env.STATE_FILE : "/code/genesis-state";
const wasm_file = process.env.WASM_FILE ? process.env.WASM_FILE : "/code/genesis-wasm";
const address = process.env.COLLATOR_ADDR ? process.env.COLLATOR_ADDR : "ws://10.0.0.2:9944";
const paraID = process.env.PARA_ID ? process.env.PARA_ID : 2001;
const acalaParaID = process.env.ACALA_PARA_ID ? process.env.PARA_ID : 2000;
const acala_state_file = process.env.STATE_FILE_ACALA ? process.env.STATE_FILE_ACALA: "/code/genesis-state-acala";
const acala_wasm_file = process.env.WASM_FILE_ACALA ? process.env.WASM_FILE_ACALA : "/code/genesis-wasm-acala";

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


async function main() {
  console.info("address is:" + address);
  const wsProvider = new WsProvider(address);
  const api = await ApiPromise.create({
    provider: wsProvider,
  });

  let keyring = new Keyring({ type: "sr25519" });
  const alice = keyring.addFromUri('//Alice');
  const bob = keyring.addFromUri('//Bob');
  console.info("get para id");

  const nextParaIdBefore = new BN(
    await (await api.query.registrar.nextFreeParaId()).toString()
  );
  console.info("submit candidancy");
  await api.tx.phragmenElection.submitCandidacy((await api.query.phragmenElection.candidates()).length
  ).signAndSend(alice);
  await wait_for_new_block(api);
  const stakeAmount = new BN(Math.pow(10, 16).toString());
  console.info("voting");
  await api.tx.phragmenElection.vote([alice.address], stakeAmount).signAndSend(bob);

  //wait 3 mins.
  console.info("waiting 3 mins")
  await new Promise((resolve) => {
    setTimeout(resolve, 180 * 1000);
  });
  await wait_for_new_block(api);
  console.info("Registering two parachain slots ");
  await api.tx.registrar.reserve().signAndSend(alice);
  await wait_for_new_block(api);
  await api.tx.registrar.reserve().signAndSend(alice);
  console.info(`Parachain slot ${paraID} registered`);
  let nextParaId = new BN(
    await (await api.query.registrar.nextFreeParaId()).toString()
  );
  let stillNotInParaId = nextParaId.toNumber() < paraID || nextParaId.toNumber() < acalaParaID;
  while (stillNotInParaId) {
    await api.tx.registrar.reserve().signAndSend(alice);
    nextParaId = new BN(
      await (await api.query.registrar.nextFreeParaId()).toString()
    )
    stillNotInParaId = nextParaId.toNumber() < paraID || nextParaId.toNumber() < acalaParaID;
    console.info(`Registering more slots Acala: ${acalaParaID}, mga: ${paraID}, current: ${nextParaId.toNumber()}`);
  }
  await wait_for_new_block(api);
  await wait_for_new_block(api);
  console.info("get para id 4");
  const requestedNextParaIdAfter = new BN(
    await (await api.query.registrar.nextFreeParaId()).toString()
  );
  const genesis = fs
    .readFileSync(state_file)
    .toString();
  const wasm = fs.readFileSync(wasm_file).toString();

  const genesisAcala = fs
    .readFileSync(acala_state_file)
    .toString();
  const wasmAcala = fs.readFileSync(acala_wasm_file).toString();
  console.info("get para id 5");
  
  const scheduleParaInit = api.tx.registrar.register(
    new BN(paraID),
    genesis,
    wasm,
  );
  console.info("registering para id");
  await scheduleParaInit.signAndSend(alice);
  await wait_for_new_block(api);
  const scheduleParaInitAcala = api.tx.registrar.register(
    new BN(acalaParaID),
    genesisAcala,
    wasmAcala,
  );
  console.info("registering para id");
  await scheduleParaInitAcala.signAndSend(alice);

  console.info("waiting 4 mins")
  await new Promise((resolve) => {
    setTimeout(resolve, 240 * 1000);
  });

  const councilProposal = api.tx.council.propose(1, api.tx.slots.forceLease(paraID, alice.address, stakeAmount, 0, 999), 64);
  const councilProposalAcala = api.tx.council.propose(1, api.tx.slots.forceLease(acalaParaID, alice.address, stakeAmount, 0, 999), 64);
  await councilProposal.signAndSend(alice);
  await wait_for_new_block(api);
  await councilProposalAcala.signAndSend(alice)


};


main().catch(console.error).finally(() => process.exit());
