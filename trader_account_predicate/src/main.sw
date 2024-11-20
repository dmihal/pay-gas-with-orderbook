predicate;

use std::{
    address::Address,
    asset_id::AssetId,
    auth::predicate_address,
    constants::ZERO_B256,
    inputs::{input_asset_id, input_coin_owner, input_amount, input_count},
    primitive_conversions::u256::*,
    outputs::{output_asset_id, output_asset_to, output_amount, output_count},
};

configurable {
    PACKET_CONTRACT_ID: ContractId = ContractId::from(ZERO_B256),
    OWNER: Address = Address::from(ZERO_B256),
}

struct CoinAmount {
    asset_id: AssetId,
    amount: u64,
    positive: bool,
}

struct CoinAccumulator {
    pub coins: Vec<(AssetId, u64)>,
}

const INDENT: u64 = 9223372036854775808u64;

impl CoinAccumulator {
    fn new(capacity: u64) -> Self {
        Self {
            coins: Vec::with_capacity(capacity),
        }
    }

    fn adjust(ref mut self, asset_id: AssetId, amount: u64, increase: bool) {
        let mut i = 0;
        while i < self.coins.len() {
            let coin = self.coins.get(i).unwrap();
            if coin.0 == asset_id {
                if increase {
                    self.coins.set(i, (coin.0, coin.1 + amount));
                } else {
                    self.coins.set(i, (coin.0, coin.1 - amount));
                }
                return;
            }
            i += 1;
        }
        let init_amount = if increase { amount + INDENT } else { INDENT - amount };
        self.coins.push((asset_id, init_amount));
    }

    fn get(self, asset_id: AssetId) -> Option<u64> {
        let mut i = 0;
        while i < self.coins.len() {
            let coin = self.coins.get(i).unwrap();
            if coin.0 == asset_id {
                return Some(coin.1);
            }
            i += 1;
        }
        None
    }

    fn assert_gte(self, asset_id: AssetId, amount: u64) {
        let amt = self.get(asset_id).unwrap();
        assert(amt >= amount + INDENT);
    }

    fn assert_lte(self, asset_id: AssetId, amount: u64) {
        let amt = self.get(asset_id).unwrap();
        assert(amt <= amount + INDENT);
    }
}

fn assert_amount(coin_limits: Vec<CoinAmount>, asset_id: AssetId, actual_amount: u64) {
    let mut i = 0;
    while i < coin_limits.len() {
        let coin_limit = coin_limits.get(i).unwrap();
        if coin_limit.asset_id == asset_id {
            if coin_limit.positive {
                let adjusted_amount = coin_limit.amount + INDENT;
                assert(actual_amount <= adjusted_amount);
            } else {
                let adjusted_amount = INDENT - coin_limit.amount;
                assert(actual_amount >= adjusted_amount);
            };
        }
        i += 1;
    }
    revert(0);
}

fn validate_signature(coin_limit: Vec<CoinAmount>, packet_id: u64, witness_idx: u64) {
    // TODO
}

fn main(
    // TODO: remove assetId from input
    coin_limit: Vec<CoinAmount>,
    witness_idx: u64,
) -> bool {
    let predicate_addr = predicate_address().unwrap();

    // TODO: withdrawals

    let num_inputs: u64 = input_count().into();
    let num_outputs: u64 = output_count().into();

    let mut acc = CoinAccumulator::new(num_inputs);

    let packet_asset_id = AssetId::new(PACKET_CONTRACT_ID, predicate_addr.into());
    let mut packet_id: u64 = u64::max();

    let mut i = 0;
    while i < num_inputs {
        if input_coin_owner(i).unwrap_or(Address::from(ZERO_B256)) == predicate_addr {
            let asset_id = input_asset_id(i).unwrap();
            let amount = input_amount(i).unwrap();

            if asset_id == packet_asset_id {
                packet_id = i;
            } else {
                // spent.adjust(asset_id, amount, true);
            }
        }
    }

    i = 0;
    while i < num_outputs {
        if output_asset_to(i).unwrap_or(Address::from(ZERO_B256)) == predicate_addr {
            let asset_id = output_asset_id(i).unwrap();
            let amount = output_amount(i);

            if asset_id != packet_asset_id {
                // spent.adjust(asset_id, amount, false);
            }
        }
    }

    i = 0;
    while i < acc.coins.len() {
        let coin = acc.coins.get(i).unwrap();
        let asset_id = coin.0;
        let amount = coin.1;

        assert_amount(coin_limit, asset_id, amount);
    }

    validate_signature(coin_limit, packet_id, witness_idx);

    true
}
