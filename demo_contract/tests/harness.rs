use fuels::{prelude::*, types::ContractId};
use fuels::{
    accounts::{
        predicate::Predicate,
        wallet::{WalletUnlocked},
    },
    core::{
        codec::EncoderConfig,
    },
    prelude::*,
    types::{
        output::Output,
        input::Input,
    },
};

abigen!(
    Predicate(
        name = "LimitOrderPredicate",
        abi = "./limit-order-predicate/out/debug/limit_order_predicate-abi.json"
    ),
    Contract(
        name = "DemoContract",
        abi = "./demo_contract/out/debug/demo_contract-abi.json"
    ),
);

pub const ETH_ASSET: AssetId = AssetId::new([0u8; 32]);
pub const USDC_ASSET: AssetId = AssetId::new([1u8; 32]);

pub async fn get_wallets() -> Vec<WalletUnlocked> {
    let num_wallets = 3;
    let initial_amount = 10_000_000_000_000_000;

    let asset_ids = [
        ETH_ASSET,
        USDC_ASSET,
    ];
    let asset_configs = asset_ids
        .map(|id| AssetConfig {
            id,
            num_coins: 1,
            coin_amount: initial_amount,
        })
        .into();

    let wallets_config = WalletsConfig::new_multiple_assets(num_wallets, asset_configs);

    let wallets = launch_custom_provider_and_get_wallets(wallets_config, None, None).await.unwrap();

    wallets
}

pub fn get_order_predicate<T: Account>(
    owner: &T,
    offer_asset: AssetId,
    receive_asset: AssetId,
    price_numerator: u64,
    price_denominator: u64,
    provider: &Provider,
) -> Predicate {
    let price = price_numerator * 1000000000 / price_denominator;
    let configurables = LimitOrderPredicateConfigurables::new(EncoderConfig::default())
        .with_PRICE(price.into()).unwrap()
        .with_OFFER_ASSET(offer_asset).unwrap()
        .with_RECEIVE_ASSET(receive_asset).unwrap()
        .with_OWNER(owner.address().into()).unwrap();

    let predicate_data = get_predicate_data(vec![], vec![], vec![]);

    let mut predicate: Predicate =
        Predicate::load_from("../limit-order-predicate/out/debug/limit_order_predicate.bin")
            .unwrap()
            .with_data(predicate_data)
            .with_configurables(configurables);
    predicate.set_provider(provider.clone());

    predicate
}

pub fn get_predicate_data(
    input_offer_coins: Vec<u64>,
    sold_output_coins: Vec<u64>,
    output_returned_offer_coins: Vec<u64>,
) -> Vec<u8> {
    LimitOrderPredicateEncoder::encode_data(
        &LimitOrderPredicateEncoder::default(),
        input_offer_coins,
        sold_output_coins,
        output_returned_offer_coins,
    ).unwrap()
}

async fn get_contract_instance(wallet: &WalletUnlocked) -> DemoContract<WalletUnlocked> {

    let id = Contract::load_from(
        "./out/debug/demo_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(wallet, TxPolicies::default())
    .await
    .unwrap();

    DemoContract::new(id.clone(), wallet.clone())
}

#[tokio::test]
async fn can_get_contract_id() {
    let wallets = get_wallets().await;
    let deployer = wallets[0].clone();
    let market_maker = wallets[1].clone();
    let user = wallets[2].clone();
    let provider = deployer.provider().unwrap();

    let contract = get_contract_instance(&deployer).await;

    let predicate = get_order_predicate(
        &market_maker,
        ETH_ASSET,
        USDC_ASSET,
        2,
        1,
        &provider,
    );

    // Initiate order
    market_maker
        .transfer(
            predicate.address(),
            1_000_000_000_000,
            ETH_ASSET,
            TxPolicies::default(),
        )
        .await
        .unwrap();

    let mut tb = contract
        .methods()
        .test_function()
        .transaction_builder()
        .await
        .unwrap();

    let mut inputs = tb.inputs().clone();
    let mut outputs = tb.outputs().clone();

    let send_amount = 100;
    let receive_amount = 50;

    let predicate_data = get_predicate_data(
        vec![1], // ETH input is idx 0
        vec![3], // USDC output is idx 2
        vec![4], // ETH returned output is idx 3
    );
    let predicate = predicate.with_data(predicate_data);
    
    let eth_inputs = predicate
        .get_asset_inputs_for_amount(ETH_ASSET, receive_amount)
        .await
        .unwrap();
    inputs.extend(eth_inputs.clone());
    
    let eth_change_amount = eth_inputs[0].amount().unwrap() - receive_amount;

    let usdc_inputs = user
        .get_asset_inputs_for_amount(USDC_ASSET, send_amount)
        .await
        .unwrap();

    let usdc_change_amount = usdc_inputs[0].amount().unwrap() - send_amount;

    inputs.extend(usdc_inputs);

    let output_coins = vec![
        Output::Change {
            to: user.address().into(),
            amount: 0,
            asset_id: ETH_ASSET,
        },
        Output::Coin {
            to: user.address().into(),
            amount: usdc_change_amount,
            asset_id: USDC_ASSET,
        },
        Output::Coin {
            to: market_maker.address().into(),
            amount: send_amount,
            asset_id: USDC_ASSET,
        },
        Output::Coin {
            to: predicate.address().into(),
            amount: eth_change_amount,
            asset_id: ETH_ASSET,
        },
    ];
    outputs.extend(output_coins);

    let mut tb = tb.with_inputs(inputs).with_outputs(outputs);

    let _ = tb.add_signer(user.clone());

    let script_transaction = tb.build(&provider).await.unwrap();

    let result = provider
        .send_transaction_and_await_commit(script_transaction)
        .await
        .unwrap();
    println!("{:#?}", result);

    // let logs = tx.decode_logs_with_type::<HelloWorld>().unwrap();
    // assert_eq!(logs.len(), 1);
}
