
// // Storage items

// // 1 to store metadata: Period Length, Offset, Timeout amount, Sell_token threshold amounts as BtreeMap
// // 1 Map to store struct for accounts containing timed out token value and the last timeout block

//     pub struct TimeoutMetadataInfo<BlockNumber>{
//         pub period_length: BlockNumber,
//         pub period_offset: BlockNumber,
//         pub timeout_amount: Balance,
//         pub swap_value_threshold: BtreeMap<TokenId, Balance>,
//         pub bootstrap_value_threshold: BtreeMap<TokenId, Balance>
//     }

//     #[pallet::storage]
// 	#[pallet::getter(fn get_timeout_metadata)]
// 	pub type TimeoutMetadata<T: Config> =
// 		StorageValue<_, TimeoutMetadataInfo<T::BlockNumber>, OptionQuery>;

//     pub struct AccountTimeoutData<BlockNumber: Default>{
//         pub total_timeout_amount: Balance,
//         pub last_timeout_block: BlockNumber,
//     }

//     #[pallet::storage]
// 	#[pallet::getter(fn get_account_timeout_data)]
// 	pub type AccountTimeoutData<T: Config> =
//         StorageMap<_, Blake2_256, AccountId, AccountTimeoutData<T::BlockNumber>, ValueQuery>;

// // Extrinsics

// // to update the pallet metadata

// // Struct that implements 
// // all handling corollary to withdraw and deposit