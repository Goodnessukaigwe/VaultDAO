#![cfg(test)]

use crate::types::{RetryConfig, ThresholdStrategy, VelocityConfig};
use crate::{InitConfig, VaultDAO, VaultDAOClient};
use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address, Env, Symbol, Vec};

fn default_init_config(env: &Env, admin: &Address) -> InitConfig {
    let mut signers = Vec::new(env);
    signers.push_back(admin.clone());

    InitConfig {
        signers,
        threshold: 1,
        quorum: 0,
        default_voting_deadline: 0,
        spending_limit: 1000,
        daily_limit: 5000,
        weekly_limit: 10000,
        timelock_threshold: 500,
        timelock_delay: 100,
        velocity_limit: VelocityConfig {
            limit: 100,
            window: 3600,
        },
        threshold_strategy: ThresholdStrategy::Fixed,
        pre_execution_hooks: Vec::new(env),
        post_execution_hooks: Vec::new(env),
        veto_addresses: Vec::new(env),
        retry_config: RetryConfig {
            enabled: false,
            max_retries: 0,
            initial_backoff_ledgers: 0,
        },
        recovery_config: crate::types::RecoveryConfig::default(env),
        staking_config: crate::types::StakingConfig::default(),
    }
}

/// Test: list_recurring_payment_ids returns empty vec when no payments exist.
#[test]
fn test_list_recurring_payment_ids_empty() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultDAO, ());
    let client = VaultDAOClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &default_init_config(&env, &admin));

    let ids = client.list_recurring_payment_ids(&0u64, &10u64);
    assert_eq!(ids.len(), 0);
}

/// Test: list_recurring_payments returns empty vec when no payments exist.
#[test]
fn test_list_recurring_payments_empty() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultDAO, ());
    let client = VaultDAOClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &default_init_config(&env, &admin));

    let payments = client.list_recurring_payments(&0u64, &10u64);
    assert_eq!(payments.len(), 0);
}

/// Test: list_recurring_payment_ids returns all IDs in ascending order.
#[test]
fn test_list_recurring_payment_ids_ascending_order() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultDAO, ());
    let client = VaultDAOClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &default_init_config(&env, &admin));

    // Create token
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token = token_contract.address();

    let recipient = Address::generate(&env);

    // Create three recurring payments
    let id1 = client.schedule_payment(
        &admin,
        &recipient,
        &token,
        &100i128,
        &Symbol::new(&env, "payment1"),
        &1000u64,
    );
    let id2 = client.schedule_payment(
        &admin,
        &recipient,
        &token,
        &200i128,
        &Symbol::new(&env, "payment2"),
        &1000u64,
    );
    let id3 = client.schedule_payment(
        &admin,
        &recipient,
        &token,
        &300i128,
        &Symbol::new(&env, "payment3"),
        &1000u64,
    );

    let ids = client.list_recurring_payment_ids(&0u64, &10u64);
    assert_eq!(ids.len(), 3);
    assert_eq!(ids.get(0).unwrap(), id1);
    assert_eq!(ids.get(1).unwrap(), id2);
    assert_eq!(ids.get(2).unwrap(), id3);
}

/// Test: list_recurring_payments returns full payment objects with correct data.
#[test]
fn test_list_recurring_payments_returns_full_objects() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultDAO, ());
    let client = VaultDAOClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &default_init_config(&env, &admin));

    // Create token
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token = token_contract.address();

    let recipient = Address::generate(&env);

    // Create a recurring payment
    let id = client.schedule_payment(
        &admin,
        &recipient,
        &token,
        &500i128,
        &Symbol::new(&env, "payment"),
        &1000u64,
    );

    let payments = client.list_recurring_payments(&0u64, &10u64);
    assert_eq!(payments.len(), 1);

    let payment = payments.get(0).unwrap();
    assert_eq!(payment.id, id);
    assert_eq!(payment.recipient, recipient);
    assert_eq!(payment.amount, 500i128);
    assert_eq!(payment.payment_count, 0);
}

/// Test: Pagination - offset and limit work correctly for recurring payments.
#[test]
fn test_list_recurring_payments_pagination() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultDAO, ());
    let client = VaultDAOClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &default_init_config(&env, &admin));

    // Create token
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token = token_contract.address();

    let recipient = Address::generate(&env);

    // Create 5 recurring payments with fixed symbols
    client.schedule_payment(
        &admin,
        &recipient,
        &token,
        &100i128,
        &Symbol::new(&env, "pay001"),
        &1000u64,
    );
    client.schedule_payment(
        &admin,
        &recipient,
        &token,
        &200i128,
        &Symbol::new(&env, "pay002"),
        &1000u64,
    );
    client.schedule_payment(
        &admin,
        &recipient,
        &token,
        &300i128,
        &Symbol::new(&env, "pay003"),
        &1000u64,
    );
    client.schedule_payment(
        &admin,
        &recipient,
        &token,
        &400i128,
        &Symbol::new(&env, "pay004"),
        &1000u64,
    );
    client.schedule_payment(
        &admin,
        &recipient,
        &token,
        &500i128,
        &Symbol::new(&env, "pay005"),
        &1000u64,
    );

    // First page: offset=0, limit=2
    let page1 = client.list_recurring_payment_ids(&0u64, &2u64);
    assert_eq!(page1.len(), 2);

    // Second page: offset=2, limit=2
    let page2 = client.list_recurring_payment_ids(&2u64, &2u64);
    assert_eq!(page2.len(), 2);

    // Third page: offset=4, limit=2
    let page3 = client.list_recurring_payment_ids(&4u64, &2u64);
    assert_eq!(page3.len(), 1);

    // Offset beyond total -> empty
    let page4 = client.list_recurring_payment_ids(&10u64, &2u64);
    assert_eq!(page4.len(), 0);
}

/// Test: list_recurring_payments returns payments in deterministic ascending order by ID.
#[test]
fn test_list_recurring_payments_ordering() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultDAO, ());
    let client = VaultDAOClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &default_init_config(&env, &admin));

    // Create token
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token = token_contract.address();

    let recipient = Address::generate(&env);

    // Create payments in random order (not sequential IDs)
    let _id3 = client.schedule_payment(
        &admin,
        &recipient,
        &token,
        &300i128,
        &Symbol::new(&env, "payment3"),
        &1000u64,
    );
    let _id1 = client.schedule_payment(
        &admin,
        &recipient,
        &token,
        &100i128,
        &Symbol::new(&env, "payment1"),
        &1000u64,
    );
    let _id2 = client.schedule_payment(
        &admin,
        &recipient,
        &token,
        &200i128,
        &Symbol::new(&env, "payment2"),
        &1000u64,
    );

    // Verify ascending order regardless of creation order
    let payments = client.list_recurring_payments(&0u64, &10u64);
    assert_eq!(payments.len(), 3);

    // IDs should be in ascending order (1, 2, 3)
    assert_eq!(payments.get(0).unwrap().id, 1);
    assert_eq!(payments.get(1).unwrap().id, 2);
    assert_eq!(payments.get(2).unwrap().id, 3);
}
