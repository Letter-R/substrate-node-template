use frame_support::{migration::storage_key_iter, pallet_prelude::*, StoragePrefixedMap};

use crate::*;

#[derive(
	Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen, Clone, Copy, PartialEq, Eq, Default,
)]
pub struct OldKitty {
	pub dna: [u8; 16],
	pub name: [u8; 4],
}

pub fn migrate<T: Config>() -> Weight {
	let on_chain_version = Pallet::<T>::on_chain_storage_version();
	let current_version = Pallet::<T>::current_storage_version();

	assert!((on_chain_version == 1) && (current_version == 2));

	let module = Kitties::<T>::module_prefix();
	let item = Kitties::<T>::storage_prefix();

	for (index, kitty) in
		storage_key_iter::<KittyId, OldKitty, Blake2_128Concat>(module, item).drain()
	{
		let new_kitty = Kitty { dna: kitty.dna, name: *b"abcdefgh" };

		Kitties::<T>::insert(index, &new_kitty);
	}
	Weight::zero()
}
