#![cfg_attr(not(feature = "std"), no_std)]
use extrinsic_info_runtime_api::runtime_api::ExtrinsicInfoRuntimeApi;
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::SeedableRng;
use sp_api::{ApiExt, ApiRef, Encode, HashT, ProvideRuntimeApi, TransactionOutcome};
use sp_core::crypto::Ss58Codec;
use sp_core::H256;
use sp_runtime::generic::BlockId;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT};
use sp_runtime::AccountId32;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::collections::vec_deque::VecDeque;
use sp_std::vec::Vec;

/// shuffles extrinsics assuring that extrinsics signed by single account will be still evaluated
/// in proper order
pub fn shuffle<'a, Block, Api>(
    api: &ApiRef<'a, Api::Api>,
    block_id: &BlockId<Block>,
    extrinsics: Vec<Block::Extrinsic>,
    hash: H256,
) -> Vec<Block::Extrinsic>
where
    Block: BlockT,
    Api: ProvideRuntimeApi<Block> + 'a,
    Api::Api: ExtrinsicInfoRuntimeApi<Block>,
{
    log::debug!(target: "block_shuffler", "shuffling extrinsics with seed: {:#X}", hash);

    let mut grouped_extrinsics: BTreeMap<Option<AccountId32>, VecDeque<_>> = extrinsics
        .into_iter()
        .fold(BTreeMap::new(), |mut groups, tx| {
            let tx_hash = BlakeTwo256::hash(&tx.encode());
            let who = api.execute_in_transaction(|api| {
                // store deserialized data and revert state modification caused by 'get_info' call
                match api.get_info(block_id, tx.clone()){
                    Ok(result) => TransactionOutcome::Rollback(Ok(result)),
                    Err(_) => TransactionOutcome::Rollback(Err(()))
                }
            })
            .expect("extrinsic deserialization should not fail!")
            .map(|info| Some(info.who)).unwrap_or(None);

            log::debug!(target: "block_shuffler", "who:{:48}  extrinsic:{:?}",who.clone().map(|x| x.to_ss58check()).unwrap_or_else(|| String::from("None")), tx_hash);
            
            groups.entry(who).or_insert(VecDeque::new()).push_back(tx);
            groups
        });

    // generate exact number of slots for each account
    // [ Alice, Alice, Alice, ... , Bob, Bob, Bob, ... ]
    let mut slots: Vec<_> = grouped_extrinsics
        .iter()
        .map(|(who, txs)| vec![who; txs.len()])
        .flatten()
        .map(|elem| elem.to_owned())
        .collect();

    // shuffle slots
    let mut seed: StdRng = SeedableRng::from_seed(hash.to_fixed_bytes());
    slots.shuffle(&mut seed);

    // fill slots using extrinsics in order
    // [ Alice, Bob, ... , Alice, Bob ]
    //              ↓↓↓
    // [ AliceExtrinsic1, BobExtrinsic1, ... , AliceExtrinsicN, BobExtrinsicN ]
    let shuffled_extrinsics: Vec<_> = slots
        .into_iter()
        .map(|who| {
            grouped_extrinsics
                .get_mut(&who)
                .unwrap()
                .pop_front()
                .unwrap()
        })
        .collect();

    log::debug!(target: "block_shuffler", "shuffled order");
    for tx in shuffled_extrinsics.iter() {
        let tx_hash = BlakeTwo256::hash(&tx.encode());
        log::debug!(target: "block_shuffler", "extrinsic:{:?}", tx_hash);
    }

    shuffled_extrinsics
}
