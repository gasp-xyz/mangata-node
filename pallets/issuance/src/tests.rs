use super::*;
use mock::{new_test_ext, roll_to_while_minting, BlocksPerRound, Issuance, System, Test, Tokens};
use sp_runtime::SaturatedConversion;

#[test]
fn linear_issuance_works() {
	new_test_ext().execute_with(|| {
		let session_number = System::block_number().saturated_into::<u32>() / BlocksPerRound::get();
		let session_issuance = <Issuance as GetIssuance>::get_all_issuance(session_number)
			.expect("session issuance is always populated in advance");
		let block_issuance = (session_issuance.0 + session_issuance.1 + session_issuance.2) /
			(BlocksPerRound::get().saturated_into::<u128>());

		// Mint in block 1
		// We are not minting in block 0, but that's okay
		assert_eq!(450044, (session_issuance.0 + session_issuance.1 + session_issuance.2));
		assert_eq!(90008, block_issuance);
		orml_tokens::MultiTokenCurrencyAdapter::<Test>::mint(0u32.into(), &1u128, block_issuance)
			.unwrap();

		roll_to_while_minting(22218, Some(90008));

		assert_eq!(3999797744, Tokens::total_issuance(0u32) as Balance);

		// This the point where the last tokens of the session will be minted after which the next session's issuance will be calculated
		// on the basis of total_issuance
		roll_to_while_minting(22219, Some(90008));

		assert_eq!(3999887752, Tokens::total_issuance(0u32) as Balance);
	});
}

#[test]
fn linear_issuance_doesnt_change_upon_burn() {
	new_test_ext().execute_with(|| {
		// Mint in block 1
		// We are not minting in block 0, but that's okay
		orml_tokens::MultiTokenCurrencyAdapter::<Test>::mint(0u32.into(), &1u128, 90008).unwrap();

		roll_to_while_minting(15000, Some(90008));

		orml_tokens::MultiTokenCurrencyAdapter::<Test>::burn_and_settle(
			0u32.into(),
			&0u128,
			100_000_000,
		)
		.unwrap();

		assert_eq!(3250120000, Tokens::total_issuance(0u32) as Balance);

		roll_to_while_minting(22218, Some(90008));

		assert_eq!(3899797744, Tokens::total_issuance(0u32) as Balance);

		// This the point where the last tokens of the session will be minted after which the next session's issuance will be calculated
		// on the basis of total_issuance
		roll_to_while_minting(22219, Some(90008));

		assert_eq!(3899887752, Tokens::total_issuance(0u32) as Balance);
	});
}

#[test]
fn issuance_stops_upon_reaching_cap() {
	new_test_ext().execute_with(|| {
		// Mint in block 1
		// We are not minting in block 0, but that's okay
		orml_tokens::MultiTokenCurrencyAdapter::<Test>::mint(0u32.into(), &1u128, 90008).unwrap();

		// This the point where the last tokens of the session will be minted after which the next session's issuance will be calculated
		// on the basis of total_issuance
		roll_to_while_minting(22219, Some(90008));

		assert_eq!(3999887752, Tokens::total_issuance(0u32) as Balance);

		// At this point the entirety of the missing issuance will be allocated to the next session

		roll_to_while_minting(22224, Some(22449));

		assert_eq!(3999999997, Tokens::total_issuance(0u32) as Balance);

		// Now there is not enough missing issuance to issue so no more mga will be issued

		roll_to_while_minting(23000, Some(0));

		assert_eq!(3999999997, Tokens::total_issuance(0u32) as Balance);
	});
}

#[test]
fn issuance_does_not_stop_upon_burn() {
	new_test_ext().execute_with(|| {
		// Mint in block 1
		// We are not minting in block 0, but that's okay
		orml_tokens::MultiTokenCurrencyAdapter::<Test>::mint(0u32.into(), &1u128, 90008).unwrap();

		// This the point where the last tokens of the session will be minted after which the next session's issuance will be calculated
		// on the basis of total_issuance
		roll_to_while_minting(22219, Some(90008));

		assert_eq!(3999887752, Tokens::total_issuance(0u32) as Balance);

		roll_to_while_minting(22221, Some(22449));

		orml_tokens::MultiTokenCurrencyAdapter::<Test>::burn_and_settle(
			0u32.into(),
			&0u128,
			100_000,
		)
		.unwrap();

		// At this point the entirety of the missing issuance will be allocated to the next session

		roll_to_while_minting(22224, Some(22449));

		assert_eq!(3999899997, Tokens::total_issuance(0u32) as Balance);

		// Now there is not enough missing issuance to issue so no more mga will be issued

		roll_to_while_minting(22229, Some(20000));

		assert_eq!(3999999997, Tokens::total_issuance(0u32) as Balance);

		roll_to_while_minting(24001, Some(0));

		assert_eq!(3999999997, Tokens::total_issuance(0u32) as Balance);
	});
}

#[test]
fn issuance_restarts_upon_burn() {
	new_test_ext().execute_with(|| {
		// Mint in block 1
		// We are not minting in block 0, but that's okay
		orml_tokens::MultiTokenCurrencyAdapter::<Test>::mint(0u32.into(), &1u128, 90008).unwrap();

		// This the point where the last tokens of the session will be minted after which the next session's issuance will be calculated
		// on the basis of total_issuance
		roll_to_while_minting(22219, Some(90008));

		assert_eq!(3999887752, Tokens::total_issuance(0u32) as Balance);

		// At this point the entirety of the missing issuance will be allocated to the next session

		roll_to_while_minting(22224, Some(22449));

		assert_eq!(3999999997, Tokens::total_issuance(0u32) as Balance);

		// Now there is not enough missing issuance to issue so no more mga will be issued

		roll_to_while_minting(23002, Some(0));

		assert_eq!(3999999997, Tokens::total_issuance(0u32) as Balance);

		orml_tokens::MultiTokenCurrencyAdapter::<Test>::burn_and_settle(
			0u32.into(),
			&0u128,
			100_000,
		)
		.unwrap();

		assert_eq!(3999899997, Tokens::total_issuance(0u32) as Balance);

		roll_to_while_minting(23004, Some(0));

		roll_to_while_minting(23009, Some(20000));

		assert_eq!(3999999997, Tokens::total_issuance(0u32) as Balance);

		roll_to_while_minting(24001, Some(0));

		assert_eq!(3999999997, Tokens::total_issuance(0u32) as Balance);
	});
}

#[test]
fn issuance_after_linear_never_execeeds_linear() {
	new_test_ext().execute_with(|| {
		// Mint in block 1
		// We are not minting in block 0, but that's okay
		orml_tokens::MultiTokenCurrencyAdapter::<Test>::mint(0u32.into(), &1u128, 90008).unwrap();

		// This the point where the last tokens of the session will be minted after which the next session's issuance will be calculated
		// on the basis of total_issuance
		roll_to_while_minting(22219, Some(90008));

		assert_eq!(3999887752, Tokens::total_issuance(0u32) as Balance);

		// At this point the entirety of the missing issuance will be allocated to the next session

		roll_to_while_minting(22224, Some(22449));

		assert_eq!(3999999997, Tokens::total_issuance(0u32) as Balance);

		// Now there is not enough missing issuance to issue so no more mga will be issued

		roll_to_while_minting(23002, Some(0));

		assert_eq!(3999999997, Tokens::total_issuance(0u32) as Balance);

		orml_tokens::MultiTokenCurrencyAdapter::<Test>::burn_and_settle(
			0u32.into(),
			&0u128,
			100_000,
		)
		.unwrap();

		assert_eq!(3999899997, Tokens::total_issuance(0u32) as Balance);

		roll_to_while_minting(23004, Some(0));

		roll_to_while_minting(23009, Some(20000));

		assert_eq!(3999999997, Tokens::total_issuance(0u32) as Balance);

		roll_to_while_minting(23023, Some(0));

		assert_eq!(3999999997, Tokens::total_issuance(0u32) as Balance);

		orml_tokens::MultiTokenCurrencyAdapter::<Test>::burn_and_settle(
			0u32.into(),
			&0u128,
			100_000_000,
		)
		.unwrap();

		assert_eq!(3899999997, Tokens::total_issuance(0u32) as Balance);

		roll_to_while_minting(23024, Some(0));

		roll_to_while_minting(23051, Some(90008));

		assert_eq!(3902430213, Tokens::total_issuance(0u32) as Balance);
	});
}
