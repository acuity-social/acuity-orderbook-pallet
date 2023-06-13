use codec::Codec;
use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;

use std::sync::Arc;

pub use pallet_acuity_orderbook::AssetId;
pub use pallet_acuity_orderbook::PriceValue;
pub use pallet_acuity_orderbook_rpc_runtime_api::OrderbookApi as OrderbookRuntimeApi;

#[rpc(client, server)]
pub trait OrderbookApi<AssetId, AccountId, PriceValue, BlockHash> {
    #[method(name = "orderbook_getPairSellers")]
    fn get_pair_sellers(
        &self,
        sell_asset_id: AssetId,
        buy_asset_id: AssetId,
        offset: u32,
        count: u32,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<AccountId>>;
    #[method(name = "orderbook_getPairSellersOrders")]
    fn get_pair_sellers_orders(
        &self,
        sell_asset_id: AssetId,
        buy_asset_id: AssetId,
        offset: u32,
        count: u32,
        at: Option<BlockHash>,
    ) -> RpcResult<(Vec<AccountId>, Vec<PriceValue>)>;
}

pub struct Orderbook<C, P> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<P>,
}

impl<C, P> Orderbook<C, P> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

/// Error type of this RPC api.
pub enum Error {
    /// The transaction was not decodable.
    DecodeError,
    /// The call to runtime failed.
    RuntimeError,
}

impl From<Error> for i32 {
    fn from(e: Error) -> i32 {
        match e {
            Error::RuntimeError => 1,
            Error::DecodeError => 2,
        }
    }
}

impl<C, AccountId, Block>
    OrderbookApiServer<AssetId, AccountId, PriceValue, <Block as BlockT>::Hash>
    for Orderbook<C, Block>
where
    AccountId: Codec,
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: OrderbookRuntimeApi<Block, AssetId, AccountId, PriceValue>,
{
    fn get_pair_sellers(
        &self,
        sell_asset_id: AssetId,
        buy_asset_id: AssetId,
        offset: u32,
        count: u32,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<AccountId>> {
        let api = self.client.runtime_api();
        let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_pair_sellers(at_hash, sell_asset_id, buy_asset_id, offset, count)
            .map_err(|e| {
                CallError::Custom(ErrorObject::owned(
                    Error::RuntimeError.into(),
                    "Unable to query dispatch info.",
                    Some(e.to_string()),
                ))
                .into()
            })
    }

    fn get_pair_sellers_orders(
        &self,
        sell_asset_id: AssetId,
        buy_asset_id: AssetId,
        offset: u32,
        count: u32,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<(Vec<AccountId>, Vec<PriceValue>)> {
        let api = self.client.runtime_api();
        let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_pair_sellers_orders(at_hash, sell_asset_id, buy_asset_id, offset, count)
            .map_err(|e| {
                CallError::Custom(ErrorObject::owned(
                    Error::RuntimeError.into(),
                    "Unable to query dispatch info.",
                    Some(e.to_string()),
                ))
                .into()
            })
    }
}
