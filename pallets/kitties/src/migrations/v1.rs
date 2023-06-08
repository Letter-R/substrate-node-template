use frame_support::{pallet_prelude::*, StoragePrefixedMap};
use frame_system::pallet_prelude::*;

use frame_support::traits::{Currency, ExistenceRequirement, Randomness};
use sp_io::hashing::blake2_128;
use sp_runtime::traits::AccountIdConversion;

use frame_support::{migration::storage_key_iter, PalletId};

use crate::*;

#[derive(
	Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen, Clone, Copy, PartialEq, Eq, Default,
)]
pub struct OldKitty(pub [u8; 16]);

pub fn migrate<T: Config>() -> Weight {
	let on_chain_version = Pallet::<T>::on_chain_storage_version();
	let current_version = Pallet::<T>::current_storage_version();

	assert!((on_chain_version == 0) && (current_version == 1));

	let module = Kitties::<T>::module_prefix();
	let item = Kitties::<T>::storage_prefix();

	for (index, kitty) in
		storage_key_iter::<KittyId, OldKitty, Blake2_128Concat>(module, item).drain()
	{
		let new_kitty = Kitty { dna: kitty.0, name: *b"abcd" };

		Kitties::<T>::insert(index, &new_kitty);
	}
	Weight::zero()
}
