
import { Keyring } from '@polkadot/keyring';
import { ApiPromise, WsProvider } from '@polkadot/api';

import BN from "bn.js";
import fs from "fs";

const state_file = process.env.STATE_FILE ? process.env.STATE_FILE : "/code/genesis-state";
const wasm_file = process.env.WASM_FILE ? process.env.WASM_FILE : "/code/genesis-wasm";
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

    const wsProvider = new WsProvider(address);
    const api = await ApiPromise.create({
      provider: wsProvider,
    });

    let keyring = new Keyring({type: "sr25519"});
    const alice = keyring.addFromUri('//Alice');
    console.info("get para id");
    const nextParaIdBefore = new BN(
      await (await api.query.registrar.nextFreeParaId()).toString()
    );
	await wait_for_new_block(api);
    console.info(`get para id 3 ${nextParaIdBefore}`);
    if ( !nextParaIdBefore.eqn(2000) ){
         console.info("Registering parachain slot 2000");
         await api.tx.registrar.reserve().signAndSend(alice);
         console.info("Parachain slot 2000 registered");
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

    console.info("get para id 5");
    const scheduleParaInit = api.tx.parasSudoWrapper.sudoScheduleParaInitialize(
      new BN(2000),
      {
        genesisHead: genesis,
        validationCode: wasm,
        parachain: true,
      }
    );
    console.info("registering para id");
    await api.tx.sudo.sudo(scheduleParaInit).signAndSend(alice);
  };


main().catch(console.error).finally(() => process.exit());
