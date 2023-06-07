#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {

	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	use frame_support::traits::{Currency, ExistenceRequirement, Randomness};
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::AccountIdConversion;

	use frame_support::PalletId;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type Currency: Currency<Self::AccountId>;
		#[pallet::constant]
		type KittyPrice: Get<BalanceOf<Self>>;
		type PalletId: Get<PalletId>;
	}

	// Kitties define
	#[derive(Encode, Decode)] // allow trans over net
	#[derive(RuntimeDebug)] //allow printed to console
	#[derive(TypeInfo)] // SCALE's type info
	#[derive(MaxEncodedLen)] //encode need know the length
	#[derive(Clone, Copy, PartialEq, Eq, Default)]
	pub struct Kitty(pub [u8; 16]);

	//
	pub type KittyId = u32;
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	// KittyId runtime storage
	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T> = StorageValue<_, KittyId, ValueQuery>;
	// Kitties runtime storage
	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyId, Kitty>;
	// KittyOwner runtime storage
	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, T::AccountId>;
	// KittyParent runtime storage
	#[pallet::storage]
	#[pallet::getter(fn kitty_parents)]
	pub type KittyParents<T: Config> =
		StorageMap<_, Blake2_128Concat, KittyId, (KittyId, KittyId), OptionQuery>;
	//
	#[pallet::storage]
	#[pallet::getter(fn kitty_on_sale)]
	pub type KittyOnSale<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, T::AccountId>;

	// Pallets use events to inform users when important changes are made.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyBreed { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyTransferred { who: T::AccountId, recipient: T::AccountId, kitty_id: KittyId },
		KittyOnSale { who: T::AccountId, kitty_id: KittyId },
		KittyBought { who: T::AccountId, kitty_id: KittyId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		InvalidKittyId,
		SameKittyId,
		NotOwner,
		AlreadyOnSale,
		AlreadyOwned,
		NotOnSale,
		LowBalance,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// create a kitty
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let kitty_id = Self::get_next_id()?;
			let kitty = Kitty(Self::random_value(&who));

			Kitties::<T>::insert(kitty_id, kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			let price = T::KittyPrice::get();
			let balance = T::Currency::free_balance(&who);
			ensure!(price < balance, Error::<T>::LowBalance);
			T::Currency::transfer(
				&who,
				&Self::get_account_id(),
				price,
				ExistenceRequirement::KeepAlive,
			)?;

			Self::deposit_event(Event::KittyCreated { who, kitty_id, kitty });

			Ok(())
		}

		/// bred a kitty
		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: KittyId,
			kitty_id_2: KittyId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);
			ensure!(Kitties::<T>::contains_key(kitty_id_1), Error::<T>::InvalidKittyId);
			ensure!(Kitties::<T>::contains_key(kitty_id_2), Error::<T>::InvalidKittyId);

			let kitty_id = Self::get_next_id()?;
			let kitty_1 = Self::kitties(kitty_id_1).unwrap();
			let kitty_2 = Self::kitties(kitty_id_2).unwrap();

			let selector = Self::random_value(&who);
			let mut data = [0u8; 16];
			for i in 0..kitty_1.0.len() {
				data[i] = (kitty_1.0[i] & selector[i]) | (kitty_2.0[i] & selector[i]);
			}
			let kitty = Kitty(data);

			let price = T::KittyPrice::get();
			let balance = T::Currency::free_balance(&who);
			ensure!(price < balance, Error::<T>::LowBalance);
			T::Currency::transfer(
				&who,
				&Self::get_account_id(),
				price,
				ExistenceRequirement::KeepAlive,
			)?;

			Kitties::<T>::insert(kitty_id, kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			KittyParents::<T>::insert(kitty_id, (kitty_id_1, kitty_id_2));
			NextKittyId::<T>::set(kitty_id + 1);

			Self::deposit_event(Event::KittyBreed { who, kitty_id, kitty });

			Ok(())
		}

		/// transfer a kitty
		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			kitty_id: KittyId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let owner = Self::kitty_owner(kitty_id).unwrap();
			ensure!(who == owner, Error::<T>::NotOwner);

			KittyOwner::<T>::insert(kitty_id, &recipient);

			Self::deposit_event(Event::KittyTransferred { who, recipient, kitty_id });

			Ok(())
		}

		/// sale a kitty
		#[pallet::call_index(3)]
		#[pallet::weight(0)]
		pub fn sale(origin: OriginFor<T>, kitty_id: KittyId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::kitties(kitty_id).ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;

			ensure!(Self::kitty_owner(kitty_id) == Some(who.clone()), Error::<T>::NotOwner);
			ensure!(Self::kitty_on_sale(kitty_id).is_none(), Error::<T>::AlreadyOnSale);

			KittyOnSale::<T>::insert(kitty_id, &who);
			Self::deposit_event(Event::KittyOnSale { who, kitty_id });

			Ok(())
		}

		/// buy a kitty
		/// input
		#[pallet::call_index(4)]
		#[pallet::weight(0)]
		pub fn buy(origin: OriginFor<T>, kitty_id: KittyId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::kitties(kitty_id).ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;

			let owner = Self::kitty_owner(kitty_id).unwrap();
			ensure!(owner != who, Error::<T>::AlreadyOwned);
			ensure!(Self::kitty_on_sale(kitty_id).is_some(), Error::<T>::NotOnSale);

			let price = T::KittyPrice::get();
			let balance = T::Currency::free_balance(&who);
			ensure!(price < balance, Error::<T>::LowBalance);
			T::Currency::transfer(&who, &owner, price, ExistenceRequirement::KeepAlive)?;

			KittyOwner::<T>::insert(kitty_id, &who);
			KittyOnSale::<T>::remove(kitty_id);

			Self::deposit_event(Event::KittyBought { who, kitty_id });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// runtime NextKittyId +=1
		fn get_next_id() -> Result<KittyId, DispatchError> {
			NextKittyId::<T>::try_mutate(|next_id| {
				let current_id = *next_id;
				*next_id = next_id
					.checked_add(1)
					.ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;

				Ok(current_id)
			})
		}

		/// get a random [u8; 16]
		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);
			payload.using_encoded(blake2_128)
		}

		///
		fn get_account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}
}
