use fuels::{
    accounts::{
        predicate::Predicate,
        wallet::{WalletUnlocked},
    },
    core::{
        codec::EncoderConfig,
    },
    prelude::*,
};

abigen!(Predicate(
    name = "LimitOrderPredicate",
    abi = "limit-order-predicate/out/debug/limit_order_predicate-abi.json"
));

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
        Predicate::load_from("./out/debug/limit_order_predicate.bin")
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
