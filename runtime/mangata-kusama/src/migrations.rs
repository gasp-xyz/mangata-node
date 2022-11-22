use super::*;

mod deprecated {
	use frame_support::sp_runtime::RuntimeDebug;

	use super::*;

	/// An active voter.
	#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, TypeInfo)]
	pub struct Voter<AccountId, Balance> {
		/// The members being backed.
		pub votes: Vec<AccountId>,
		/// The amount of stake placed on this vote.
		pub stake: Balance,
		/// The amount of deposit reserved for this vote.
		///
		/// To be unreserved upon removal.
		pub deposit: Balance,
	}

	impl<AccountId, Balance: Default> Default for Voter<AccountId, Balance> {
		fn default() -> Self {
			Self { votes: vec![], stake: Default::default(), deposit: Default::default() }
		}
	}

	/// A holder of a seat as either a member or a runner-up.
	#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, TypeInfo)]
	pub struct SeatHolder<AccountId, Balance> {
		/// The holder.
		pub who: AccountId,
		/// The total backing stake.
		pub stake: Balance,
		/// The amount of deposit held on-chain.
		///
		/// To be unreserved upon renouncing, or slashed upon being a loser.
		pub deposit: Balance,
	}
}

pub mod phragmen_elections {
	#[cfg(feature = "try-runtime")]
	use frame_support::{
		migration::{have_storage_value, storage_key_iter},
		Twox64Concat,
	};
	use frame_support::{storage::unhashed::clear_prefix, traits::OnRuntimeUpgrade};
	use sp_io::hashing::twox_128;

	use super::*;

	pub struct PhragmenElectionsMigration;
	impl OnRuntimeUpgrade for PhragmenElectionsMigration {
		fn on_runtime_upgrade() -> Weight {
			log::info!(
				target: "phragmen_elections",
				"on_runtime_upgrade: Attempted to apply phragmen_elections migration"
			);

			let module_name = "Elections";

			let clear_result = clear_prefix(&twox_128(module_name.as_bytes()), None, None);
			log::info!(
				target: "phragmen_elections",
				"clear_result: {:?}, {:?}",
				clear_result.maybe_cursor, clear_result.loops
			);

			<Runtime as frame_system::Config>::DbWeight::get().reads_writes(
				(clear_result.loops as u64) + 1_u64,
				(clear_result.loops as u64) + 1_u64,
			)
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<(), &'static str> {
			log::info!(
				target: "phragmen_elections",
				"pre_upgrade check: phragmen_elections"
			);

			assert!(have_storage_value(b"Elections", b"Members", b"",));

			assert!(have_storage_value(b"Elections", b"RunnersUp", b"",));

			assert!(have_storage_value(b"Elections", b"Candidates", b"",));

			assert!(have_storage_value(b"Elections", b"ElectionRounds", b"",));

			assert!(storage_key_iter::<
				<Runtime as frame_system::Config>::AccountId,
				deprecated::Voter<<Runtime as frame_system::Config>::AccountId, Balance>,
				Twox64Concat,
			>(b"Elections", b"Voting",)
			.next()
			.is_some());

			assert!(have_storage_value(b"Elections", b":__STORAGE_VERSION__:", b"",));

			Ok(())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade() -> Result<(), &'static str> {
			log::info!(
				target: "phragmen_elections",
				"post_upgrade check: phragmen_elections"
			);

			assert!(!have_storage_value(b"Elections", b"Members", b"",));

			assert!(!have_storage_value(b"Elections", b"RunnersUp", b"",));

			assert!(!have_storage_value(b"Elections", b"Candidates", b"",));

			assert!(!have_storage_value(b"Elections", b"ElectionRounds", b"",));

			assert!(storage_key_iter::<
				<Runtime as frame_system::Config>::AccountId,
				deprecated::Voter<<Runtime as frame_system::Config>::AccountId, Balance>,
				Twox64Concat,
			>(b"Elections", b"Voting",)
			.next()
			.is_none());

			assert!(!have_storage_value(b"Elections", b":__STORAGE_VERSION__:", b"",));

			Ok(())
		}
	}
}
