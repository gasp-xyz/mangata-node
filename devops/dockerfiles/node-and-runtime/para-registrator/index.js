
import { Keyring } from '@polkadot/keyring';
import { ApiPromise, WsProvider } from '@polkadot/api';

import BN from "bn.js";
import fs from "fs";



async function main () {
    const wsUrl = "ws://10.0.0.2:9944";
    const paraId = 2000;
    const pathToFiles = "/tmp/";

    const wsProvider = new WsProvider(wsUrl);
    const api = await ApiPromise.create({
      provider: wsProvider,
    });

    let keyring = new Keyring({type: "sr25519"});
    const alice = keyring.addFromUri('//Alice');
    // sudo = new User(keyring, sudoUserName);
    // testUser1 = new User(keyring, user);
    console.info("get para id");
    // add users to pair.
    // keyring.addPair(testUser1.keyRingPair);
    // keyring.addPair(sudo.keyRingPair);
    //await signTx(api, api.tx.registrar.reserve(), testUser1.keyRingPair);
    const nextParaIdBefore = new BN(
      await (await api.query.registrar.nextFreeParaId()).toString()
    );
    console.info(`get para id 3 ${nextParaIdBefore}`);
    if ( !nextParaIdBefore.eqn(2000) ){
        try {
         console.info("Registering parachain slot 2000");
          await api.tx.registrar.reserve().signAndSend(testUser1.keyRingPair);
         console.info("Parachain slot 2000 registered");
        } catch (error) {}
    }

    console.info("get para id 4");
    const requestedNextParaIdAfter = new BN(
      await (await api.query.registrar.nextFreeParaId()).toString()
    );
    const genesis = fs
      .readFileSync("/code/genesis-state")
      .toString();
    const wasm = fs.readFileSync("/code/genesis-wasm").toString();

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
    const unsub = await api.tx.sudo.sudo(scheduleParaInit).signAndSend(alice, async ({ events = [] , status, dispatchError }) => {
        console.info(status.toHuman());
    });

    //         const unsub = await tx.signAndSend(this.signer, options, async ({ events = [] , status, dispatchError }) => {
    //             if (status.isInBlock || status.isFinalized) {
    // console.info(result.toHuman());
    // console.info("para id registered");
    // // await waitNewBlock();
  };


main().catch(console.error).finally(() => process.exit());
