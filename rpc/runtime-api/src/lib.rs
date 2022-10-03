#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
use sp_std::prelude::Vec;

sp_api::decl_runtime_apis! {
    pub trait OrderbookApi<AssetId, AccountId, PriceValue> where
        AssetId: codec::Codec,
		AccountId: codec::Codec,
        PriceValue: codec::Codec,
    {
        fn get_pair_sellers(sell_asset_id: AssetId, buy_asset_id: AssetId, offset: u32, count: u32) -> sp_std::prelude::Vec<AccountId>;
        fn get_pair_sellers_orders(sell_asset_id: AssetId, buy_asset_id: AssetId, offset: u32, count: u32) -> (sp_std::prelude::Vec<AccountId>, sp_std::prelude::Vec<PriceValue>);
    }
}
