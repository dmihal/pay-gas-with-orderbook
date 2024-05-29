mod lib;

use fuels::{
    prelude::*,
    types::{
        errors::Error,
        errors::transaction::Reason,
        output::Output,
    },
};
use lib::{ETH_ASSET, USDC_ASSET, get_predicate_data, get_wallets, get_order_predicate};

#[tokio::test]
async fn can_swap() {
    let wallets = get_wallets().await;
    let owner = wallets[0].clone();
    let buyer = wallets[1].clone();
    let provider = owner.provider().unwrap();

    let predicate = get_order_predicate(
        &owner,
        ETH_ASSET,
        USDC_ASSET,
        2,
        1,
        &provider,
    );

    // Initiate order
    owner
        .transfer(
            predicate.address(),
            1_000_000_000_000,
            ETH_ASSET,
            TxPolicies::default(),
        )
        .await
        .unwrap();

    // Execute order

    let send_amount = 100;
    let receive_amount = 50;

    let predicate_data = get_predicate_data(
        vec![0], // ETH input is idx 0
        vec![2], // USDC output is idx 2
        vec![3], // ETH returned output is idx 3
    );
    let predicate = predicate.with_data(predicate_data);
    
    let eth_inputs = predicate
        .get_asset_inputs_for_amount(ETH_ASSET, receive_amount)
        .await
        .unwrap();
    let mut inputs = eth_inputs.clone();
    
    let eth_change_amount = eth_inputs[0].amount().unwrap() - receive_amount;

    let usdc_inputs = buyer
        .get_asset_inputs_for_amount(USDC_ASSET, send_amount)
        .await
        .unwrap();

    let usdc_change_amount = usdc_inputs[0].amount().unwrap() - send_amount;

    inputs.extend(usdc_inputs);

    let outputs = vec![
        Output::Change {
            to: buyer.address().into(),
            amount: 0,
            asset_id: ETH_ASSET,
        },
        Output::Coin {
            to: buyer.address().into(),
            amount: usdc_change_amount,
            asset_id: USDC_ASSET,
        },
        Output::Coin {
            to: owner.address().into(),
            amount: send_amount,
            asset_id: USDC_ASSET,
        },
        Output::Coin {
            to: predicate.address().into(),
            amount: eth_change_amount,
            asset_id: ETH_ASSET,
        },
    ];

    // Create the Tx
    let mut transaction_builder = ScriptTransactionBuilder::prepare_transfer(
        inputs,
        outputs,
        TxPolicies::default().with_max_fee(10_000).with_script_gas_limit(0),
    );

    let _ = transaction_builder.add_signer(buyer.clone());

    let script_transaction = transaction_builder.build(&provider).await.unwrap();

    provider
        .send_transaction_and_await_commit(script_transaction)
        .await
        .unwrap();
}

#[tokio::test]
async fn cant_swap_excessive_output() {
    let wallets = get_wallets().await;
    let owner = wallets[0].clone();
    let buyer = wallets[1].clone();
    let provider = owner.provider().unwrap();

    let predicate = get_order_predicate(
        &owner,
        ETH_ASSET,
        USDC_ASSET,
        2,
        1,
        &provider,
    );

    // Initiate order
    owner
        .transfer(
            predicate.address(),
            1_000_000_000_000,
            ETH_ASSET,
            TxPolicies::default(),
        )
        .await
        .unwrap();

    // Execute order

    let send_amount = 100;
    let receive_amount = 51;

    let predicate_data = get_predicate_data(
        vec![0], // ETH input is idx 0
        vec![2], // USDC output is idx 2
        vec![3], // ETH returned output is idx 3
    );
    let predicate = predicate.with_data(predicate_data);
    
    let eth_inputs = predicate
        .get_asset_inputs_for_amount(ETH_ASSET, receive_amount)
        .await
        .unwrap();
    let mut inputs = eth_inputs.clone();
    
    let eth_change_amount = eth_inputs[0].amount().unwrap() - receive_amount;

    let usdc_inputs = buyer
        .get_asset_inputs_for_amount(USDC_ASSET, send_amount)
        .await
        .unwrap();

    let usdc_change_amount = usdc_inputs[0].amount().unwrap() - send_amount;

    inputs.extend(usdc_inputs);

    let outputs = vec![
        Output::Change {
            to: buyer.address().into(),
            amount: 0,
            asset_id: ETH_ASSET,
        },
        Output::Coin {
            to: buyer.address().into(),
            amount: usdc_change_amount,
            asset_id: USDC_ASSET,
        },
        Output::Coin {
            to: owner.address().into(),
            amount: send_amount,
            asset_id: USDC_ASSET,
        },
        Output::Coin {
            to: predicate.address().into(),
            amount: eth_change_amount,
            asset_id: ETH_ASSET,
        },
    ];

    // Create the Tx
    let mut transaction_builder = ScriptTransactionBuilder::prepare_transfer(
        inputs,
        outputs,
        TxPolicies::default().with_max_fee(10_000).with_script_gas_limit(0),
    );

    let _ = transaction_builder.add_signer(buyer.clone());

    let script_transaction = transaction_builder.build(&provider).await.unwrap();

    let err = provider
        .send_transaction_and_await_commit(script_transaction)
        .await
        .err()
        .expect("Transaction should fail");

    if let Error::Transaction(ref reason) = err {
        if let Reason::Validation(reason_str) = reason {
            assert_eq!(reason_str, "PredicateVerificationFailed(Panic(PredicateReturnedNonOne))");
        } else {
            panic!("Unexpected error: {:#?}", err);
        }
    } else {
        panic!("Unexpected error: {:#?}", err);
    };
}
