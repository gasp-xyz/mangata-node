
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_module, decl_storage,
    weights::DispatchClass,
};
use sp_inherents::{InherentData, InherentIdentifier, ProvideInherent, IsFatalError};
use sp_runtime::RuntimeString;
use sp_std::result;
use sp_core::H256;
use codec::Encode;
use codec::Decode;

#[cfg(feature = "std")]
use sp_inherents::ProvideInherentData;

/// The module configuration trait
pub trait Trait: frame_system::Trait {
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        #[weight =  (
            10_000,
            DispatchClass::Mandatory
        )]
        fn set(origin, seed: H256) {
            <Self as Store>::Seed::put(seed);
        }

    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RandomSeed {
        /// Current time for the current block.
        pub Seed get(fn seed) : H256;
    }
}

impl<T: Trait> Module<T> {
    pub fn get() -> H256 {
        Self::seed()
    }
}

// originally in sp-module
pub type RandomSeedInherentType = H256;
const RANDOM_SEED_INHERENT_IDENTIFIER: InherentIdentifier = *b"blckseed";

#[derive(Encode, sp_runtime::RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Decode))]
pub enum RandomSeedInherentError {
	Other(RuntimeString),
}


impl RandomSeedInherentError {
	/// Try to create an instance ouf of the given identifier and data.
	#[cfg(feature = "std")]
	pub fn try_from(id: &InherentIdentifier, data: &[u8]) -> Option<Self> {
		if id == &RANDOM_SEED_INHERENT_IDENTIFIER {
			<RandomSeedInherentError as codec::Decode>::decode(&mut &data[..]).ok()
		} else {
			None
		}
	}
}

impl IsFatalError for RandomSeedInherentError {
	fn is_fatal_error(&self) -> bool {
        true
	}
}

fn extract_inherent_data(data: &InherentData) -> Result<RandomSeedInherentType, RuntimeString> {
    data.get_data::<RandomSeedInherentType>(&RANDOM_SEED_INHERENT_IDENTIFIER)
        .map_err(|_| RuntimeString::from("Invalid random seed inherent data encoding."))?
        .ok_or_else(|| "Random Seed inherent data is not provided.".into())
}

#[cfg(feature = "std")]
pub struct RandomSeedInherentDataProvider(pub H256);

#[cfg(feature = "std")]
impl ProvideInherentData for RandomSeedInherentDataProvider {
	fn inherent_identifier(&self) -> &'static InherentIdentifier {
		&RANDOM_SEED_INHERENT_IDENTIFIER
	}

	fn provide_inherent_data(
		&self,
		inherent_data: &mut InherentData,
	) -> Result<(), sp_inherents::Error> {
        inherent_data.put_data(RANDOM_SEED_INHERENT_IDENTIFIER, &self.0)
	}

	fn error_to_string(&self, error: &[u8]) -> Option<String> {
		RandomSeedInherentError::try_from(&RANDOM_SEED_INHERENT_IDENTIFIER, error).map(|e| format!("{:?}", e))
	}
}


impl<T: Trait> ProvideInherent for Module<T> {
    type Call = Call<T>;
    type Error = RandomSeedInherentError;
    const INHERENT_IDENTIFIER: InherentIdentifier = *b"blckseed";

    fn create_inherent(data: &InherentData) -> Option<Self::Call> {
        log::debug!(target: "rand-seed", "initializing random seed");
        let seed: H256 = extract_inherent_data(data)
            .expect("Gets and decodes random seed");
        Some(Call::set(seed))
    }

    fn check_inherent(_call: &Self::Call, _data: &InherentData) -> result::Result<(), Self::Error> {
        Ok(())
    }
}

