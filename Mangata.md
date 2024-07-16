# Block construction/execution
Mangata implements front-running bots prevention on automated market maker crypto exchange. For that reason it was decided to split block production and block execution into two following blocks. Execution of the transaction included in block N is delayed by 1 block comparing to origin substrate implementation. It is following block producer (N+1) who calculates the order of transactions execution from the previous block. This way none is able to foresee in what order transactions will be performed. 

Affected creates:
- [sc-block-builder](https://docs.rs/sc-block-builder/0.8.0/sc_block_builder/index.html) - execution of extrinsics from previous block on block creation

# Shuffling
Extrinsics are randomly shuffled but still preserve original order per every account - motivation for such algorithm was the fact that following transactions from a given account may depend on other transactions from the same account.


Origin order:
```
| Alice TX1 | Alice TX2 |  Alice TX3 |  Bob TX1 | Bob TX2 |
```
example shuffled order:

```
| Alice TX1 |  Bob TX1  |  Alice TX2 |  Bob TX2 | Alice TX3 |
```

As shuffling occurs on block execution not creation, every node needs to calculate the same order and agree on storage state afterwards. That is achieved using Fisher-Yates shuffling algorithm with Xoshiro256++ as PRNG initialized with seed stored in blockchain storage.  Usage of sr25519 key pair together with VRF capabilities guarantees that block producer cannot impact execution order and it's easy to validate if the seed was properly calculated by validators/following nodes (thanks to VRF verification that only requires block author public key). Seed itself is stored into the storage and is publically accessible through dedicated runtime API [RandomSeedApi](https://github.com/mangata-finance/mangata-node/blob/59b8e6d27c76f89cddbad777ffbeafd1d7f86297/pallets/random-seed/runtime-api/src/lib.rs).

PRNG initialization seed for block `N-1` is signature of 'input message' signed with block producer private key where 'input message' is defined as:
- Seed from block `N-1`
- babe epoch randomness (changed in every epoch)

There is no way for block producer to manipulate seed value in order to impact execution order of extrinsics - also block producer is not able to calculate seed for a block he is going to author sooner than N-1 block is produced.

Seed value is injected into the block as [InherentData](https://docs.rs/sp-inherents/2.0.0/sp_inherents/struct.InherentData.html) by Babe consensus protocol. Whether block producer finds out it's his turn to create a block new seed is being calculated. Seeds value is validated (VRF verify) by other nodes when new block is being imported - in case of seed calculation violation block will be rejected.


Affected crates
- [sc-block-builder](https://docs.rs/sc-block-builder/0.8.0/sc_block_builder/index.html) - extracting seed value on inherents creation
- [sc-basic-authorship](https://docs.rs/sc-basic-authorship/0.8.0/sc_basic_authorship/index.html) - shuffling extrinsics before passing them to block builder
- [sc-consensus-babe](https://docs.rs/sc-basic-authorship/0.8.0/sc_basic_authorship/index.html) - calculating and injecting shuffling seed value into InherentData, shuffling seed verification
- [sc-service](https://docs.rs/sc-service/0.8.0/sc_service/index.html) - fetching seed value and extrinsic shuffling for 'following nodes'
#Tokens

Mangata uses concept of liquidity tokens. One can create a liquidity pool(together with liquidity pool token) using a pair of tokens and then use the tokens of that pool to stake. New currency trait [`MultiTokenCurrency`](https://github.com/mangata-finance/mangata-node/blob/0846c42a7b7fd29e19fd1b30043ddb3b55a8f250/pallets/tokens/src/multi_token_currency.rs#L14) was introduced  and integrated with [staking pallet](https://github.com/mangata-finance/mangata-node/tree/59b8e6d27c76f89cddbad777ffbeafd1d7f86297/pallets/staking). Origin implementation was not sufficient as it only works with single, statically defined currency in our case we require supporting multiple dynamically created currencies.


Affected crates
- [orml-tokens](https://docs.rs/orml-tokens/0.3.1/orml_tokens/index.html) - Introduced new `MultiTokenCurrency` trait, using it as `Balances` replacement
- [pallet-staking](https://docs.rs/pallet-staking/2.0.0/pallet_staking/index.html) - Alining with `MultiTokenCurrency` trait.

#JS API
Because of shuffled delayed transaction execution same applies to events triggered by extrinsics. Origin JS API is not sufficient to map extrinsics to triggered events. Link between transaction <-> event is done based on transaction/events order. In Mangata order of extrinsics inside the block differs from their actual execution order. For that reason dedicated async API methods were introduced to provide missing functionality. When transaction is sent:
1. API waits for notification that it has been included in block N
2. API fetches block N and N+1
3. API stores information about list of events produced in block N+1
3. API reads seed from block N+1 and calculates execution order of extrinsics from block N
4. API maps events to shuffled extrinsics list

from API client perspective it's as easy as 
```
  signAndWaitTx(
		api.tx.tokens.transfer(to, asset_id, amount),
        from,
        nonce
   ).then( (events) => {
      events.forEach((e) => {
          console.log(e.toHuman())
      });
  })

```
