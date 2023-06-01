use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_work() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0; // judge only
		let account_id = 1;

		assert_eq!(KittiesModule::next_kitty_id(), kitty_id);
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));
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

#[test]
fn breed_work() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;

		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));
		//assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 2);

		assert_ok!(KittiesModule::breed(RuntimeOrigin::signed(account_id), kitty_id, kitty_id + 1));
		System::assert_last_event(RuntimeEvent::KittiesModule(Event::KittyBreed {
			who: account_id,
			kitty_id: kitty_id + 2,
			kitty: KittiesModule::kitties(kitty_id + 2).unwrap(),
		}));

		let breed_kitty_id = 2;
		assert_eq!(KittiesModule::kitties(breed_kitty_id).is_some(), true);
		assert_eq!(KittiesModule::kitty_owner(breed_kitty_id), Some(account_id));
		assert_eq!(KittiesModule::kitty_parents(breed_kitty_id), Some((kitty_id, kitty_id + 1)));
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

#[test]
fn transfer_work() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;
		let recipient_id = 2;

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

		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(1)));
		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));

		assert_noop!(
			KittiesModule::transfer(RuntimeOrigin::signed(recipient_id), recipient_id, kitty_id),
			Error::<Test>::NotOwner
		);
	})
}
