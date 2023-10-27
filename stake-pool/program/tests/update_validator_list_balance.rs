#![allow(clippy::integer_arithmetic)]
#![cfg(feature = "test-sbf")]

mod helpers;

use {
    helpers::*,
    domichain_program::{borsh::try_from_slice_unchecked, program_pack::Pack, pubkey::Pubkey},
    domichain_program_test::*,
    solana_sdk::{hash::Hash, signature::Signer, stake::state::StakeState},
    spl_stake_pool::{
        state::{StakePool, StakeStatus, ValidatorList},
        MAX_VALIDATORS_TO_UPDATE, MINIMUM_RESERVE_SATOMIS,
    },
    spl_token::state::Mint,
    std::num::NonZeroU32,
};

async fn setup(
    num_validators: usize,
) -> (
    ProgramTestContext,
    Hash,
    StakePoolAccounts,
    Vec<ValidatorStakeAccount>,
    Vec<DepositStakeAccount>,
    u64,
    u64,
    u64,
) {
    let mut context = program_test().start_with_context().await;
    let first_normal_slot = context.genesis_config().epoch_schedule.first_normal_slot;
    let slots_per_epoch = context.genesis_config().epoch_schedule.slots_per_epoch;
    let mut slot = first_normal_slot;
    context.warp_to_slot(slot).unwrap();

    let reserve_stake_amount = TEST_STAKE_AMOUNT * 2 * num_validators as u64;
    let stake_pool_accounts = StakePoolAccounts::default();
    stake_pool_accounts
        .initialize_stake_pool(
            &mut context.banks_client,
            &context.payer,
            &context.last_blockhash,
            reserve_stake_amount + MINIMUM_RESERVE_SATOMIS,
        )
        .await
        .unwrap();

    // Add several accounts with some stake
    let mut stake_accounts: Vec<ValidatorStakeAccount> = vec![];
    let mut deposit_accounts: Vec<DepositStakeAccount> = vec![];
    for i in 0..num_validators {
        let stake_account = ValidatorStakeAccount::new(
            &stake_pool_accounts.stake_pool.pubkey(),
            NonZeroU32::new(i as u32),
            u64::MAX,
        );
        create_vote(
            &mut context.banks_client,
            &context.payer,
            &context.last_blockhash,
            &stake_account.validator,
            &stake_account.vote,
        )
        .await;

        let error = stake_pool_accounts
            .add_validator_to_pool(
                &mut context.banks_client,
                &context.payer,
                &context.last_blockhash,
                &stake_account.stake_account,
                &stake_account.vote.pubkey(),
                stake_account.validator_stake_seed,
            )
            .await;
        assert!(error.is_none());

        let deposit_account = DepositStakeAccount::new_with_vote(
            stake_account.vote.pubkey(),
            stake_account.stake_account,
            TEST_STAKE_AMOUNT,
        );
        deposit_account
            .create_and_delegate(
                &mut context.banks_client,
                &context.payer,
                &context.last_blockhash,
            )
            .await;

        stake_accounts.push(stake_account);
        deposit_accounts.push(deposit_account);
    }

    // Warp forward so the stakes properly activate, and deposit
    slot += slots_per_epoch;
    context.warp_to_slot(slot).unwrap();
    let last_blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .unwrap();

    stake_pool_accounts
        .update_all(
            &mut context.banks_client,
            &context.payer,
            &context.last_blockhash,
            stake_accounts
                .iter()
                .map(|v| v.vote.pubkey())
                .collect::<Vec<Pubkey>>()
                .as_slice(),
            false,
        )
        .await;

    for deposit_account in &mut deposit_accounts {
        deposit_account
            .deposit_stake(
                &mut context.banks_client,
                &context.payer,
                &last_blockhash,
                &stake_pool_accounts,
            )
            .await;
    }

    slot += slots_per_epoch;
    context.warp_to_slot(slot).unwrap();
    let last_blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .unwrap();

    stake_pool_accounts
        .update_all(
            &mut context.banks_client,
            &context.payer,
            &last_blockhash,
            stake_accounts
                .iter()
                .map(|v| v.vote.pubkey())
                .collect::<Vec<Pubkey>>()
                .as_slice(),
            false,
        )
        .await;

    let last_blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .unwrap();

    (
        context,
        last_blockhash,
        stake_pool_accounts,
        stake_accounts,
        deposit_accounts,
        TEST_STAKE_AMOUNT,
        reserve_stake_amount,
        slot,
    )
}

#[tokio::test]
async fn success_with_normal() {
    let num_validators = 5;
    let (
        mut context,
        last_blockhash,
        stake_pool_accounts,
        stake_accounts,
        _,
        validator_satomis,
        reserve_satomis,
        mut slot,
    ) = setup(num_validators).await;

    // Check current balance in the list
    let rent = context.banks_client.get_rent().await.unwrap();
    let stake_rent = rent.minimum_balance(std::mem::size_of::<StakeState>());
    let stake_pool_info = get_account(
        &mut context.banks_client,
        &stake_pool_accounts.stake_pool.pubkey(),
    )
    .await;
    let stake_pool = try_from_slice_unchecked::<StakePool>(&stake_pool_info.data).unwrap();
    let validator_list_sum = get_validator_list_sum(
        &mut context.banks_client,
        &stake_pool_accounts.reserve_stake.pubkey(),
        &stake_pool_accounts.validator_list.pubkey(),
    )
    .await;
    assert_eq!(stake_pool.total_satomis, validator_list_sum);
    // initially, have all of the deposits plus their rent, and the reserve stake
    let initial_satomis =
        (validator_satomis + stake_rent) * num_validators as u64 + reserve_satomis;
    assert_eq!(validator_list_sum, initial_satomis);

    // Simulate rewards
    for stake_account in &stake_accounts {
        context.increment_vote_account_credits(&stake_account.vote.pubkey(), 100);
    }

    // Warp one more epoch so the rewards are paid out
    let slots_per_epoch = context.genesis_config().epoch_schedule.slots_per_epoch;
    slot += slots_per_epoch;
    context.warp_to_slot(slot).unwrap();

    let last_blockhash = context
        .banks_client
        .get_new_latest_blockhash(&last_blockhash)
        .await
        .unwrap();

    stake_pool_accounts
        .update_all(
            &mut context.banks_client,
            &context.payer,
            &last_blockhash,
            stake_accounts
                .iter()
                .map(|v| v.vote.pubkey())
                .collect::<Vec<Pubkey>>()
                .as_slice(),
            false,
        )
        .await;
    let new_satomis = get_validator_list_sum(
        &mut context.banks_client,
        &stake_pool_accounts.reserve_stake.pubkey(),
        &stake_pool_accounts.validator_list.pubkey(),
    )
    .await;
    assert!(new_satomis > initial_satomis);

    let stake_pool_info = get_account(
        &mut context.banks_client,
        &stake_pool_accounts.stake_pool.pubkey(),
    )
    .await;
    let stake_pool = try_from_slice_unchecked::<StakePool>(&stake_pool_info.data).unwrap();
    assert_eq!(new_satomis, stake_pool.total_satomis);
}

#[tokio::test]
async fn merge_into_reserve() {
    let (
        mut context,
        last_blockhash,
        stake_pool_accounts,
        stake_accounts,
        _,
        satomis,
        _,
        mut slot,
    ) = setup(MAX_VALIDATORS_TO_UPDATE).await;

    let pre_satomis = get_validator_list_sum(
        &mut context.banks_client,
        &stake_pool_accounts.reserve_stake.pubkey(),
        &stake_pool_accounts.validator_list.pubkey(),
    )
    .await;

    let reserve_stake = context
        .banks_client
        .get_account(stake_pool_accounts.reserve_stake.pubkey())
        .await
        .unwrap()
        .unwrap();
    let pre_reserve_satomis = reserve_stake.satomis;

    println!("Decrease from all validators");
    for stake_account in &stake_accounts {
        let error = stake_pool_accounts
            .decrease_validator_stake(
                &mut context.banks_client,
                &context.payer,
                &last_blockhash,
                &stake_account.stake_account,
                &stake_account.transient_stake_account,
                satomis,
                stake_account.transient_stake_seed,
            )
            .await;
        assert!(error.is_none());
    }

    println!("Update, should not change, no merges yet");
    stake_pool_accounts
        .update_all(
            &mut context.banks_client,
            &context.payer,
            &last_blockhash,
            stake_accounts
                .iter()
                .map(|v| v.vote.pubkey())
                .collect::<Vec<Pubkey>>()
                .as_slice(),
            false,
        )
        .await;

    let expected_satomis = get_validator_list_sum(
        &mut context.banks_client,
        &stake_pool_accounts.reserve_stake.pubkey(),
        &stake_pool_accounts.validator_list.pubkey(),
    )
    .await;
    assert_eq!(pre_satomis, expected_satomis);

    let stake_pool_info = get_account(
        &mut context.banks_client,
        &stake_pool_accounts.stake_pool.pubkey(),
    )
    .await;
    let stake_pool = try_from_slice_unchecked::<StakePool>(&stake_pool_info.data).unwrap();
    assert_eq!(expected_satomis, stake_pool.total_satomis);

    println!("Warp one more epoch so the stakes deactivate");
    let slots_per_epoch = context.genesis_config().epoch_schedule.slots_per_epoch;
    slot += slots_per_epoch;
    context.warp_to_slot(slot).unwrap();

    let last_blockhash = context
        .banks_client
        .get_new_latest_blockhash(&last_blockhash)
        .await
        .unwrap();
    stake_pool_accounts
        .update_all(
            &mut context.banks_client,
            &context.payer,
            &last_blockhash,
            stake_accounts
                .iter()
                .map(|v| v.vote.pubkey())
                .collect::<Vec<Pubkey>>()
                .as_slice(),
            false,
        )
        .await;
    let expected_satomis = get_validator_list_sum(
        &mut context.banks_client,
        &stake_pool_accounts.reserve_stake.pubkey(),
        &stake_pool_accounts.validator_list.pubkey(),
    )
    .await;
    assert_eq!(pre_satomis, expected_satomis);

    let reserve_stake = context
        .banks_client
        .get_account(stake_pool_accounts.reserve_stake.pubkey())
        .await
        .unwrap()
        .unwrap();
    let post_reserve_satomis = reserve_stake.satomis;
    assert!(post_reserve_satomis > pre_reserve_satomis);

    let stake_pool_info = get_account(
        &mut context.banks_client,
        &stake_pool_accounts.stake_pool.pubkey(),
    )
    .await;
    let stake_pool = try_from_slice_unchecked::<StakePool>(&stake_pool_info.data).unwrap();
    assert_eq!(expected_satomis, stake_pool.total_satomis);
}

#[tokio::test]
async fn merge_into_validator_stake() {
    let (
        mut context,
        last_blockhash,
        stake_pool_accounts,
        stake_accounts,
        _,
        satomis,
        reserve_satomis,
        mut slot,
    ) = setup(MAX_VALIDATORS_TO_UPDATE).await;

    let rent = context.banks_client.get_rent().await.unwrap();
    let pre_satomis = get_validator_list_sum(
        &mut context.banks_client,
        &stake_pool_accounts.reserve_stake.pubkey(),
        &stake_pool_accounts.validator_list.pubkey(),
    )
    .await;

    // Increase stake to all validators
    let stake_rent = rent.minimum_balance(std::mem::size_of::<StakeState>());
    let current_minimum_delegation = stake_pool_get_minimum_delegation(
        &mut context.banks_client,
        &context.payer,
        &last_blockhash,
    )
    .await;
    let available_satomis =
        reserve_satomis - (stake_rent + current_minimum_delegation) * stake_accounts.len() as u64;
    let increase_amount = available_satomis / stake_accounts.len() as u64;
    for stake_account in &stake_accounts {
        let error = stake_pool_accounts
            .increase_validator_stake(
                &mut context.banks_client,
                &context.payer,
                &last_blockhash,
                &stake_account.transient_stake_account,
                &stake_account.stake_account,
                &stake_account.vote.pubkey(),
                increase_amount,
                stake_account.transient_stake_seed,
            )
            .await;
        assert!(error.is_none());
    }

    // Warp just a little bit to get a new blockhash and update again
    context.warp_to_slot(slot + 10).unwrap();
    let last_blockhash = context
        .banks_client
        .get_new_latest_blockhash(&last_blockhash)
        .await
        .unwrap();

    // Update, should not change, no merges yet
    let error = stake_pool_accounts
        .update_all(
            &mut context.banks_client,
            &context.payer,
            &last_blockhash,
            stake_accounts
                .iter()
                .map(|v| v.vote.pubkey())
                .collect::<Vec<Pubkey>>()
                .as_slice(),
            false,
        )
        .await;
    assert!(error.is_none());

    let expected_satomis = get_validator_list_sum(
        &mut context.banks_client,
        &stake_pool_accounts.reserve_stake.pubkey(),
        &stake_pool_accounts.validator_list.pubkey(),
    )
    .await;
    assert_eq!(pre_satomis, expected_satomis);
    let stake_pool_info = get_account(
        &mut context.banks_client,
        &stake_pool_accounts.stake_pool.pubkey(),
    )
    .await;
    let stake_pool = try_from_slice_unchecked::<StakePool>(&stake_pool_info.data).unwrap();
    assert_eq!(expected_satomis, stake_pool.total_satomis);

    // Warp one more epoch so the stakes activate, ready to merge
    let slots_per_epoch = context.genesis_config().epoch_schedule.slots_per_epoch;
    slot += slots_per_epoch;
    context.warp_to_slot(slot).unwrap();

    let last_blockhash = context
        .banks_client
        .get_new_latest_blockhash(&last_blockhash)
        .await
        .unwrap();
    let error = stake_pool_accounts
        .update_all(
            &mut context.banks_client,
            &context.payer,
            &last_blockhash,
            stake_accounts
                .iter()
                .map(|v| v.vote.pubkey())
                .collect::<Vec<Pubkey>>()
                .as_slice(),
            false,
        )
        .await;
    assert!(error.is_none());
    let current_satomis = get_validator_list_sum(
        &mut context.banks_client,
        &stake_pool_accounts.reserve_stake.pubkey(),
        &stake_pool_accounts.validator_list.pubkey(),
    )
    .await;
    let stake_pool_info = get_account(
        &mut context.banks_client,
        &stake_pool_accounts.stake_pool.pubkey(),
    )
    .await;
    let stake_pool = try_from_slice_unchecked::<StakePool>(&stake_pool_info.data).unwrap();
    assert_eq!(current_satomis, stake_pool.total_satomis);

    // Check that transient accounts are gone
    for stake_account in &stake_accounts {
        assert!(context
            .banks_client
            .get_account(stake_account.transient_stake_account)
            .await
            .unwrap()
            .is_none());
    }

    // Check validator stake accounts have the expected balance now:
    // validator stake account minimum + deposited satomis + rents + increased satomis
    let expected_satomis = current_minimum_delegation + satomis + increase_amount + stake_rent;
    for stake_account in &stake_accounts {
        let validator_stake =
            get_account(&mut context.banks_client, &stake_account.stake_account).await;
        assert_eq!(validator_stake.satomis, expected_satomis);
    }

    // Check reserve stake accounts for expected balance:
    // own rent, other account rents, and 1 extra satomi
    let reserve_stake = get_account(
        &mut context.banks_client,
        &stake_pool_accounts.reserve_stake.pubkey(),
    )
    .await;
    assert_eq!(
        reserve_stake.satomis,
        MINIMUM_RESERVE_SATOMIS + stake_rent * (1 + stake_accounts.len() as u64)
    );
}

#[tokio::test]
async fn merge_transient_stake_after_remove() {
    let (
        mut context,
        last_blockhash,
        stake_pool_accounts,
        stake_accounts,
        _,
        satomis,
        reserve_satomis,
        mut slot,
    ) = setup(1).await;

    let rent = context.banks_client.get_rent().await.unwrap();
    let stake_rent = rent.minimum_balance(std::mem::size_of::<StakeState>());
    let current_minimum_delegation = stake_pool_get_minimum_delegation(
        &mut context.banks_client,
        &context.payer,
        &last_blockhash,
    )
    .await;
    let deactivated_satomis = satomis;
    // Decrease and remove all validators
    for stake_account in &stake_accounts {
        let error = stake_pool_accounts
            .decrease_validator_stake(
                &mut context.banks_client,
                &context.payer,
                &last_blockhash,
                &stake_account.stake_account,
                &stake_account.transient_stake_account,
                deactivated_satomis,
                stake_account.transient_stake_seed,
            )
            .await;
        assert!(error.is_none());
        let error = stake_pool_accounts
            .remove_validator_from_pool(
                &mut context.banks_client,
                &context.payer,
                &last_blockhash,
                &stake_account.stake_account,
                &stake_account.transient_stake_account,
            )
            .await;
        assert!(error.is_none());
    }

    // Warp forward to merge time
    let slots_per_epoch = context.genesis_config().epoch_schedule.slots_per_epoch;
    slot += slots_per_epoch;
    context.warp_to_slot(slot).unwrap();

    // Update without merge, status should be DeactivatingTransient
    let error = stake_pool_accounts
        .update_all(
            &mut context.banks_client,
            &context.payer,
            &last_blockhash,
            stake_accounts
                .iter()
                .map(|v| v.vote.pubkey())
                .collect::<Vec<Pubkey>>()
                .as_slice(),
            true,
        )
        .await;
    assert!(error.is_none());

    let validator_list = get_account(
        &mut context.banks_client,
        &stake_pool_accounts.validator_list.pubkey(),
    )
    .await;
    let validator_list =
        try_from_slice_unchecked::<ValidatorList>(validator_list.data.as_slice()).unwrap();
    assert_eq!(validator_list.validators.len(), 1);
    assert_eq!(
        validator_list.validators[0].status,
        StakeStatus::DeactivatingAll
    );
    assert_eq!(
        validator_list.validators[0].active_stake_satomis,
        stake_rent + current_minimum_delegation
    );
    assert_eq!(
        validator_list.validators[0].transient_stake_satomis,
        deactivated_satomis
    );

    // Update with merge, status should be ReadyForRemoval and no satomis
    let error = stake_pool_accounts
        .update_validator_list_balance(
            &mut context.banks_client,
            &context.payer,
            &last_blockhash,
            stake_accounts
                .iter()
                .map(|v| v.vote.pubkey())
                .collect::<Vec<Pubkey>>()
                .as_slice(),
            false,
        )
        .await;
    assert!(error.is_none());

    // stake accounts were merged in, none exist anymore
    for stake_account in &stake_accounts {
        let not_found_account = context
            .banks_client
            .get_account(stake_account.stake_account)
            .await
            .unwrap();
        assert!(not_found_account.is_none());
        let not_found_account = context
            .banks_client
            .get_account(stake_account.transient_stake_account)
            .await
            .unwrap();
        assert!(not_found_account.is_none());
    }
    let validator_list = get_account(
        &mut context.banks_client,
        &stake_pool_accounts.validator_list.pubkey(),
    )
    .await;
    let validator_list =
        try_from_slice_unchecked::<ValidatorList>(validator_list.data.as_slice()).unwrap();
    assert_eq!(validator_list.validators.len(), 1);
    assert_eq!(
        validator_list.validators[0].status,
        StakeStatus::ReadyForRemoval
    );
    assert_eq!(validator_list.validators[0].stake_satomis().unwrap(), 0);

    let reserve_stake = context
        .banks_client
        .get_account(stake_pool_accounts.reserve_stake.pubkey())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        reserve_stake.satomis,
        reserve_satomis + deactivated_satomis + stake_rent * 2 + MINIMUM_RESERVE_SATOMIS
    );

    // Update stake pool balance and cleanup, should be gone
    let error = stake_pool_accounts
        .update_stake_pool_balance(&mut context.banks_client, &context.payer, &last_blockhash)
        .await;
    assert!(error.is_none());

    let error = stake_pool_accounts
        .cleanup_removed_validator_entries(
            &mut context.banks_client,
            &context.payer,
            &last_blockhash,
        )
        .await;
    assert!(error.is_none());

    let validator_list = get_account(
        &mut context.banks_client,
        &stake_pool_accounts.validator_list.pubkey(),
    )
    .await;
    let validator_list =
        try_from_slice_unchecked::<ValidatorList>(validator_list.data.as_slice()).unwrap();
    assert_eq!(validator_list.validators.len(), 0);
}

#[tokio::test]
async fn success_with_burned_tokens() {
    let num_validators = 5;
    let (
        mut context,
        last_blockhash,
        stake_pool_accounts,
        stake_accounts,
        deposit_accounts,
        _,
        _,
        mut slot,
    ) = setup(num_validators).await;

    let mint_info = get_account(
        &mut context.banks_client,
        &stake_pool_accounts.pool_mint.pubkey(),
    )
    .await;
    let mint = Mint::unpack(&mint_info.data).unwrap();

    let stake_pool_info = get_account(
        &mut context.banks_client,
        &stake_pool_accounts.stake_pool.pubkey(),
    )
    .await;
    let stake_pool = try_from_slice_unchecked::<StakePool>(&stake_pool_info.data).unwrap();
    assert_eq!(mint.supply, stake_pool.pool_token_supply);

    burn_tokens(
        &mut context.banks_client,
        &context.payer,
        &last_blockhash,
        &stake_pool_accounts.token_program_id,
        &stake_pool_accounts.pool_mint.pubkey(),
        &deposit_accounts[0].pool_account.pubkey(),
        &deposit_accounts[0].authority,
        deposit_accounts[0].pool_tokens,
    )
    .await
    .unwrap();

    let mint_info = get_account(
        &mut context.banks_client,
        &stake_pool_accounts.pool_mint.pubkey(),
    )
    .await;
    let mint = Mint::unpack(&mint_info.data).unwrap();
    assert_ne!(mint.supply, stake_pool.pool_token_supply);

    let slots_per_epoch = context.genesis_config().epoch_schedule.slots_per_epoch;
    slot += slots_per_epoch;
    context.warp_to_slot(slot).unwrap();

    stake_pool_accounts
        .update_all(
            &mut context.banks_client,
            &context.payer,
            &last_blockhash,
            stake_accounts
                .iter()
                .map(|v| v.vote.pubkey())
                .collect::<Vec<Pubkey>>()
                .as_slice(),
            false,
        )
        .await;

    let stake_pool_info = get_account(
        &mut context.banks_client,
        &stake_pool_accounts.stake_pool.pubkey(),
    )
    .await;
    let stake_pool = try_from_slice_unchecked::<StakePool>(&stake_pool_info.data).unwrap();

    assert_eq!(mint.supply, stake_pool.pool_token_supply);
}

#[tokio::test]
async fn fail_with_uninitialized_validator_list() {} // TODO

#[tokio::test]
async fn success_with_force_destaked_validator() {}
