use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
mod fn_create {
	use super::*;
	#[test]
	fn create_work() {
		new_test_ext().execute_with(|| {
			let kitty_id = 0;
			let account_id = 1;
			assert_eq!(KittiesModule::next_kitty_id(), kitty_id);

			let price = KittyPrice::get();

			assert_ok!(Balances::force_set_balance(
				RuntimeOrigin::root(),
				account_id.clone(),
				10 * price
			));

			let balance_before = Balances::free_balance(&account_id);

			assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));

			let balance_after = Balances::free_balance(account_id);

			//let event = System::events().last();
			System::assert_last_event(RuntimeEvent::KittiesModule(Event::KittyCreated {
				who: account_id,
				kitty_id,
				kitty: KittiesModule::kitties(kitty_id).unwrap(),
			}));

			assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 1);
			assert_eq!(KittiesModule::kitties(kitty_id).is_some(), true);
			assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));
			assert_eq!(KittiesModule::kitty_parents(kitty_id), None);

			// Assert that the account balance has decreased by the price
			assert_eq!(balance_before - balance_after, price);
		})
	}

	#[test]
	fn create_fail_with_max_kitty_id() {
		new_test_ext().execute_with(|| {
			let account_id = 1;
			crate::NextKittyId::<Test>::set(crate::KittyId::max_value());
			assert_noop!(
				KittiesModule::create(RuntimeOrigin::signed(account_id)),
				Error::<Test>::InvalidKittyId
			);
		})
	}
}
mod fn_breed {
	use super::*;

	#[test]
	fn breed_work() {
		new_test_ext().execute_with(|| {
			let kitty_id = 0;
			let account_id = 1;

			let price = KittyPrice::get();

			assert_ok!(Balances::force_set_balance(
				RuntimeOrigin::root(),
				account_id.clone(),
				10 * price
			));

			assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));
			assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));
			//assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 2);

			assert_ok!(KittiesModule::breed(
				RuntimeOrigin::signed(account_id),
				kitty_id,
				kitty_id + 1
			));
			System::assert_last_event(RuntimeEvent::KittiesModule(Event::KittyBreed {
				who: account_id,
				kitty_id: kitty_id + 2,
				kitty: KittiesModule::kitties(kitty_id + 2).unwrap(),
			}));

			let breed_kitty_id = 2;
			assert_eq!(KittiesModule::kitties(breed_kitty_id).is_some(), true);
			assert_eq!(KittiesModule::kitty_owner(breed_kitty_id), Some(account_id));
			assert_eq!(
				KittiesModule::kitty_parents(breed_kitty_id),
				Some((kitty_id, kitty_id + 1))
			);
		})
	}

	#[test]
	fn breed_fail_with_same_parent_id() {
		new_test_ext().execute_with(|| {
			let kitty_id = 0;
			let account_id = 1;

			assert_noop!(
				KittiesModule::breed(RuntimeOrigin::signed(account_id), kitty_id, kitty_id),
				Error::<Test>::SameKittyId
			);
		})
	}

	#[test]
	fn breed_fail_with_not_exist_id() {
		new_test_ext().execute_with(|| {
			let kitty_id = 0;
			let account_id = 1;

			assert_noop!(
				KittiesModule::breed(RuntimeOrigin::signed(account_id), kitty_id, kitty_id + 1),
				Error::<Test>::InvalidKittyId
			);
		})
	}
}

mod fn_transfer {
	use super::*;

	#[test]
	fn transfer_work() {
		new_test_ext().execute_with(|| {
			let kitty_id = 0;
			let account_id = 1;
			let recipient_id = 2;

			let price = KittyPrice::get();

			assert_ok!(Balances::force_set_balance(
				RuntimeOrigin::root(),
				account_id.clone(),
				10 * price
			));
			assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));
			assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));

			assert_ok!(KittiesModule::transfer(
				RuntimeOrigin::signed(account_id),
				recipient_id,
				kitty_id
			));
			System::assert_last_event(RuntimeEvent::KittiesModule(Event::KittyTransferred {
				who: account_id,
				recipient: recipient_id,
				kitty_id,
			}));

			assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(recipient_id));
		})
	}

	#[test]
	fn transfer_fail_with_wrong_owner() {
		new_test_ext().execute_with(|| {
			let kitty_id = 0;
			let account_id = 1;
			let recipient_id = 2;

			let price = KittyPrice::get();
			assert_ok!(Balances::force_set_balance(
				RuntimeOrigin::root(),
				account_id.clone(),
				10 * price
			));
			assert_ok!(KittiesModule::create(RuntimeOrigin::signed(1)));
			assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));

			assert_noop!(
				KittiesModule::transfer(
					RuntimeOrigin::signed(recipient_id),
					recipient_id,
					kitty_id
				),
				Error::<Test>::NotOwner
			);
		})
	}
}

mod fn_sale {
	use super::*;
	#[test]
	fn sale_work() {
		new_test_ext().execute_with(|| {
			let kitty_id = 0;
			let account_id = 1;

			let price = KittyPrice::get();
			assert_ok!(Balances::force_set_balance(
				RuntimeOrigin::root(),
				account_id.clone(),
				10 * price
			));
			assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));

			assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id));
			assert!(KittiesModule::kitty_on_sale(kitty_id).is_some());
		})
	}

	#[test]
	fn sale_fail_with_wrong_owner() {
		new_test_ext().execute_with(|| {
			let kitty_id = 0;
			let account_id_1 = 1;
			let account_id_2 = 2;

			let price = KittyPrice::get();
			assert_ok!(Balances::force_set_balance(
				RuntimeOrigin::root(),
				account_id_2.clone(),
				10 * price
			));
			assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id_2)));

			assert_noop!(
				KittiesModule::sale(RuntimeOrigin::signed(account_id_1), kitty_id),
				Error::<Test>::NotOwner
			);
		})
	}

	#[test]
	fn sale_fail_with_multi_sale() {
		new_test_ext().execute_with(|| {
			let kitty_id = 0;
			let account_id = 1;

			let price = KittyPrice::get();
			assert_ok!(Balances::force_set_balance(
				RuntimeOrigin::root(),
				account_id.clone(),
				10 * price
			));
			assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));

			assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id));
			assert_noop!(
				KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id),
				Error::<Test>::AlreadyOnSale
			);
		})
	}
}
mod fn_buy {
	use super::*;

	#[test]
	fn buy_work() {
		new_test_ext().execute_with(|| {
			let kitty_id = 0;
			let sale_account_id = 1;
			let buy_account_id = 2;

			let price = KittyPrice::get();
			assert_ok!(Balances::force_set_balance(
				RuntimeOrigin::root(),
				buy_account_id.clone(),
				10 * price
			));
			assert_ok!(Balances::force_set_balance(
				RuntimeOrigin::root(),
				sale_account_id.clone(),
				10 * price
			));
			assert_ok!(KittiesModule::create(RuntimeOrigin::signed(sale_account_id)));
			assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(sale_account_id), kitty_id));

			assert_ok!(KittiesModule::buy(RuntimeOrigin::signed(buy_account_id), kitty_id));
		});
	}

	#[test]
	fn buy_fail_with_saler_buy() {
		new_test_ext().execute_with(|| {
			let kitty_id = 0;
			let sale_account_id = 1;

			let price = KittyPrice::get();
			assert_ok!(Balances::force_set_balance(
				RuntimeOrigin::root(),
				sale_account_id.clone(),
				10 * price
			));
			assert_ok!(KittiesModule::create(RuntimeOrigin::signed(sale_account_id)));
			assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(sale_account_id), kitty_id));

			assert_noop!(
				KittiesModule::buy(RuntimeOrigin::signed(sale_account_id), kitty_id),
				Error::<Test>::AlreadyOwned
			);
		});
	}

	#[test]
	fn buy_fail_with_not_sale() {
		new_test_ext().execute_with(|| {
			let kitty_id = 0;
			let sale_account_id = 1;
			let buy_account_id = 2;

			let price = KittyPrice::get();
			assert_ok!(Balances::force_set_balance(
				RuntimeOrigin::root(),
				buy_account_id.clone(),
				10 * price
			));
			assert_ok!(Balances::force_set_balance(
				RuntimeOrigin::root(),
				sale_account_id.clone(),
				10 * price
			));
			assert_ok!(KittiesModule::create(RuntimeOrigin::signed(sale_account_id)));
			//assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(sale_account_id), kitty_id));

			assert_noop!(
				KittiesModule::buy(RuntimeOrigin::signed(buy_account_id), kitty_id),
				Error::<Test>::NotOnSale
			);
		});
	}

	#[test]
	fn buy_fail_with_low_balance() {
		new_test_ext().execute_with(|| {
			let kitty_id = 0;
			let sale_account_id = 1;
			let buy_account_id = 2;

			let price = KittyPrice::get();
			assert_ok!(Balances::force_set_balance(
				RuntimeOrigin::root(),
				buy_account_id.clone(),
				(0.5 * (price as f32)) as u128
			));
			assert_ok!(Balances::force_set_balance(
				RuntimeOrigin::root(),
				sale_account_id.clone(),
				10 * price
			));
			assert_ok!(KittiesModule::create(RuntimeOrigin::signed(sale_account_id)));
			assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(sale_account_id), kitty_id));

			assert_noop!(
				KittiesModule::buy(RuntimeOrigin::signed(buy_account_id), kitty_id),
				Error::<Test>::LowBalance
			);
		});
	}
}
