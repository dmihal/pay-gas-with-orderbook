predicate;
use std::{
    address::Address,
    asset_id::AssetId,
    auth::predicate_address,
    constants::ZERO_B256,
    hash::{
        Hash,
        sha256,
    },
    inputs::{input_asset_id, input_coin_owner, input_amount},
    primitive_conversions::u256::*,
    outputs::{output_asset_id, output_asset_to, output_amount},
    storage::storage_string::*,
    string::String,
};

configurable {
    PRICE: u256 = 0x0u256,
    RECEIVE_ASSET: AssetId = AssetId::from(ZERO_B256),
    OFFER_ASSET: AssetId = AssetId::from(ZERO_B256),
    OWNER: Address = Address::from(ZERO_B256),
}

const SCALE: u256 = 0x3b9aca00u256; // 1000000000u256;

fn main(input_offer_coins: Vec<u64>, sold_output_coins: Vec<u64>, output_returned_offer_coins: Vec<u64>) -> bool {
    let predicate_address = predicate_address();
    let current_input_idx = current_input();

    let mut buy_amount = 0;
    let mut sell_amount = 0;
    let mut current_input_validated = false;

    let mut i = 0;
    while i < input_offer_coins.len() {
        let coin_idx = input_offer_coins.get(i).unwrap();

        // if input_asset_id(coin_idx).unwrap() != OFFER_ASSET {
        //     return false;
        // }
        // if input_coin_owner(coin_idx).unwrap() != predicate_address {
        //     return false;
        // }
        if coin_idx == current_input_idx {
            current_input_validated = true;
        }

        buy_amount += input_amount(coin_idx).unwrap();
        i += 1;
    }

    // Cheap way to make sure that all predicate coins in the TX are validated in input_offer_coins
    // TODO: just iterate over inputs and check the owner?
    if !current_input_validated {
        return false;
    }

    i = 0;
    while i < output_returned_offer_coins.len() {
        let coin_idx = output_returned_offer_coins.get(i).unwrap();

    //     if output_asset_id(coin_idx).unwrap() != OFFER_ASSET {
    //         return true;
    //     }
    //     if output_asset_to(coin_idx).unwrap() != predicate_address.bits() {
    //         return true;
    //     }

        buy_amount -= output_amount(coin_idx);

        i += 1;
    }

    i = 0;
    while i < sold_output_coins.len() {
        let coin_idx = sold_output_coins.get(i).unwrap();

    //     if output_asset_id(coin_idx).unwrap() != RECEIVE_ASSET {
    //         return true;
    //     }
    //     if output_asset_to(coin_idx).unwrap() != OWNER.bits() {
    //         return true;
    //     }

        sell_amount += output_amount(coin_idx);
        i += 1;
    }

    let price = SCALE * sell_amount.into() / buy_amount.into();
    if (price < PRICE) {
        return false;
    }

    true
}

pub fn current_input() -> u64 {
    // Get index of current predicate.
    // i3 = GM_GET_VERIFYING_PREDICATE
    asm(r1) {
        gm r1 i3;
        r1: u64
    }
}
