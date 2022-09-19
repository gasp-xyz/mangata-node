
import { Keyring } from '@polkadot/keyring';
import { ApiPromise, WsProvider } from '@polkadot/api';

import BN from "bn.js";
import fs from "fs";
import { exit } from 'process';

const state_file =  "/code/genesis-state";
const wasm_file = "/code/genesis-wasm";
const address = process.env.COLLATOR_ADDR ? process.env.COLLATOR_ADDR : "ws://10.0.0.2:9944";

async function wait_for_new_block(api){
    let wait = new Promise(async (resolve, _) => {
        let counter = 0;
        const unsub_new_heads = await api.derive.chain.subscribeNewHeads(async (header) => {
            if (counter++ > 0){
                console.info(`new block produced #${header.number}`);
                unsub_new_heads()
                resolve()
            }
        });
    })

    await wait;
}


async function main () {
    console.info("address is:"+ address);
    const wsProvider = new WsProvider(address);
    const api = await ApiPromise.create({
      provider: wsProvider,
    });

    let keyring = new Keyring({type: "sr25519"});
    const alice = keyring.addFromUri('//Alice');
    const bob = keyring.addFromUri('//Bob');
    console.info("get para id");

    const nextParaIdBefore = new BN(
      await (await api.query.registrar.nextFreeParaId()).toString()
    );
    console.info("submit candidancy");
    await api.tx.phragmenElection.submitCandidacy( (await api.query.phragmenElection.candidates() ).length
    ).signAndSend(alice);
    await wait_for_new_block(api);
    const stakeAmount = new BN(Math.pow(10, 16).toString());
    console.info("voting");
    await api.tx.phragmenElection.vote( [ alice.address ] , stakeAmount).signAndSend(bob);
    
    //wait 3 mins.
    console.info("waiting 3 mins")
    await new Promise((resolve) => {
      setTimeout(resolve, 180 *1000);
    });
    
    await wait_for_new_block(api);
      console.info(`get para id 3 ${nextParaIdBefore}`);
      if (!nextParaIdBefore.eqn(2000) ){
            console.info("Registering parachain slot 2000");
            await api.tx.registrar.reserve().signAndSend(alice);
            console.info("Parachain slot 2000 registered");
      }
    await wait_for_new_block(api);
    await wait_for_new_block(api);
    await wait_for_new_block(api);
    const numberOfRegistrations = new BN(2110).sub(nextParaIdBefore).toNumber();
    const batches = [];
    for (let index = 0; index <= numberOfRegistrations; index++) {
      batches.push(api.tx.registrar.reserve())
    }
    await api.tx.utility.batchAll(batches).signAndSend(alice);
    await wait_for_new_block(api);
    await wait_for_new_block(api);
    await wait_for_new_block(api);
      await api.tx.registrar.reserve().signAndSend(alice);
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
   console.info("get para id 5");
   const scheduleParaInit = api.tx.registrar.register(
     new BN(2110),
     genesis,
     wasm,
   );
   console.info("registering para id");
   await scheduleParaInit.signAndSend(alice);

   console.info("waiting 4 mins")
   await new Promise((resolve) => {
     setTimeout(resolve, 240 *1000);
   });
   
   const councilProposal2 = api.tx.council.propose( 1 , api.tx.slots.forceLease(2110, alice.address , stakeAmount , 0 , 999), 64);
   await councilProposal2.signAndSend(alice)


 };


main().catch(console.error).finally(() => process.exit());
