// This file is part of Substrate.

// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;

pub mod v1 {
	use super::*;
	use crate::log;
	use frame_support::traits::OnRuntimeUpgrade;

	/// Trivial migration which makes the roles of each pool optional.
	///
	/// Note: The depositor is not optional since he can never change.
	pub struct MigrateToV1<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
		fn on_runtime_upgrade() -> Weight {
			let current = Pallet::<T>::current_storage_version();
			let onchain = Pallet::<T>::on_chain_storage_version();

			log!(
				info,
				"Running migration with current storage version {:?} / onchain {:?}",
				current,
				onchain
			);

			if current == 1 && onchain == 0 {
				// this is safe to execute on any runtime that has a bounded number of pools.

				if Phase::<T>::get() == BootstrapPhase::Finished && ActivePair::<T>::get().is_none()
				{
					ActivePair::<T>::put((4_u32, 0_u32));
					log!(info, "Filled ActivePair with default value");
					T::DbWeight::get().reads_writes(3, 1)
				} else {
					T::DbWeight::get().reads_writes(3, 0)
				}
			} else {
				log!(info, "Migration did not executed. This probably should be removed");
				T::DbWeight::get().reads(2)
			}
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<(), &'static str> {
			log!(info, "Bootstrap::pre_upgrade");
			assert_eq!(Pallet::<T>::on_chain_storage_version(), 1);
			Ok(())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade() -> Result<(), &'static str> {
			log!(info, "Bootstrap::post_upgrade");
			assert_eq!(Pallet::<T>::on_chain_storage_version(), 1);
			Ok(())
		}
	}
}
