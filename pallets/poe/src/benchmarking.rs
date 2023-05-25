//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;
use frame_benchmarking::{benchmarks, v2::*};
use frame_support::{traits::Get, BoundedVec};
use frame_system::RawOrigin;

benchmarks! {
	create_claim {
		let claim: BoundedVec<u8, T::MaxClaimLength> = vec![0; T::MaxClaimLength::get() as usize].try_into().unwrap();
		let caller: T::AccountId = whitelisted_caller();
	}: _(RawOrigin::Signed(caller.clone()), claim.clone())
	verify {
		assert!(Proofs::<T>::contains_key(claim));
	}

	revole_claim {
		let claim: BoundedVec<u8, T::MaxClaimLength> = vec![0; T::MaxClaimLength::get() as usize].try_into().unwrap();
		let caller: T::AccountId = whitelisted_caller();
		Proofs::<T>::insert(&claim, (caller.clone(), frame_system::Pallet::<T>::block_number()));
	}: _(RawOrigin::Signed(caller.clone()), claim.clone().try_into().unwrap())
	verify {
		assert!(!Proofs::<T>::contains_key(claim));
	}

	transfer_claim {
		let claim: BoundedVec<u8, T::MaxClaimLength> = vec![0; T::MaxClaimLength::get() as usize].try_into().unwrap();
		let caller: T::AccountId = whitelisted_caller();
		let recipient: T::AccountId = account("recipient", 0, 0);
		Proofs::<T>::insert(&claim, (caller.clone(), frame_system::Pallet::<T>::block_number()));
	}: _(RawOrigin::Signed(caller.clone()), recipient.clone(), claim.clone())
	verify {
		assert_eq!(Proofs::<T>::get(claim).unwrap().0, recipient);
	}

	impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test);
}
