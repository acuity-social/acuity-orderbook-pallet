#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use sp_runtime::RuntimeDebug;
use codec::{Encode, Decode, MaxEncodedLen};
use scale_info::TypeInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// 2 bytes chain type
// 6 bytes eth chainId
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ChainId([u8; 8]);

// 8 bytes chainId
// 2 bytes address type 0 - base, 1 - ERC20
// 2 bytes adapterId (smart contract)
// 20 bytes tokenAddress
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct AssetId([u8; 32]);

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ForeignAccount([u8; 32]);

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct PriceValue {
	pub price: u128,
	pub value: u128,
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
    #[pallet::getter(fn account_chain_id_account)]
    pub(super) type AccountForeignAccount<T: Config> = StorageDoubleMap<_,
		Blake2_128Concat, T::AccountId,
		Blake2_128Concat, ChainId,
		ForeignAccount
	>;

	#[pallet::storage]
	#[pallet::getter(fn account_pair_order)]
	pub(super) type AccountPairOrder<T: Config> = StorageNMap<
	    _,
	    (
	        NMapKey<Blake2_128Concat, T::AccountId>,	// seller
	        NMapKey<Blake2_128Concat, AssetId>,			// sell assetId
	        NMapKey<Blake2_128Concat, AssetId>,			// buy assetId
	    ),
	    PriceValue,
	>;

	#[pallet::storage]
    #[pallet::getter(fn pair_count)]
    pub(super) type PairCount<T: Config> = StorageDoubleMap<_,
		Blake2_128Concat, AssetId,			// sell assetId
		Blake2_128Concat, AssetId,			// buy assetId
        u32,
		ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn pair_account_list)]
	pub(super) type PairAccountList<T: Config> = StorageNMap<
	    _,
	    (
	        NMapKey<Blake2_128Concat, AssetId>,			// sell assetId
	        NMapKey<Blake2_128Concat, AssetId>,			// buy assetId
			NMapKey<Blake2_128Concat, u32>,				// index
	    ),
		T::AccountId,	// seller
	>;

	#[pallet::storage]
    #[pallet::getter(fn pair_account_index)]
	// Mapping of sell assetId to buy assetId to seller to index + 1 in PairAccountList.
	pub(super) type PairAccountIndex<T: Config> = StorageNMap<
	    _,
	    (
	        NMapKey<Blake2_128Concat, AssetId>,			// sell assetId
	        NMapKey<Blake2_128Concat, AssetId>,			// buy assetId
			NMapKey<Blake2_128Concat, T::AccountId>,	// seller
	    ),
		u32,	// index + 1
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An order was set. [account, chain_id, foreign_account]
		SetForeignAccount(T::AccountId, ChainId, ForeignAccount),
		/// An order was set. [sell_asset_id, buy_asset_id, price, value]
		SetOrder(AssetId, AssetId, u128, u128),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(50_000_000)]
		pub fn set_foreign_account(origin: OriginFor<T>, chain_id: ChainId, foreign_account: ForeignAccount) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            <AccountForeignAccount<T>>::insert(&sender, chain_id, foreign_account);
            Self::deposit_event(Event::SetForeignAccount(sender, chain_id, foreign_account));
			Ok(().into())
		}

		#[pallet::weight(50_000_000)]
		pub fn set_order(origin: OriginFor<T>, sell_asset_id: AssetId, buy_asset_id: AssetId, price: u128, value: u128) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
			let price_value = PriceValue{
				price: price,
				value: value,
			};

            <AccountPairOrder<T>>::insert((sender, sell_asset_id, buy_asset_id), price_value);
            Self::deposit_event(Event::SetOrder(sell_asset_id, buy_asset_id, price, value));
			Ok(().into())
		}

		#[pallet::weight(50_000_000)]
		pub fn remove_order(origin: OriginFor<T>, sell_asset_id: AssetId, buy_asset_id: AssetId) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			<AccountPairOrder<T>>::remove((sender, sell_asset_id, buy_asset_id));
			Ok(().into())
		}

		#[pallet::weight(50_000_000)]
		pub fn remove_orders_for_sell_asset(origin: OriginFor<T>, sell_asset_id: AssetId) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			<AccountPairOrder<T>>::clear_prefix((sender, sell_asset_id), u32::max_value(), None);
			Ok(().into())
		}

		#[pallet::weight(50_000_000)]
		pub fn remove_orders(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			<AccountPairOrder<T>>::clear_prefix((sender,), u32::max_value(), None);
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_pair_sellers(sell_asset_id: AssetId, buy_asset_id: AssetId, offset: u32, count: u32) -> sp_std::prelude::Vec<T::AccountId> {
			let mut sellers = sp_std::prelude::Vec::new();

			let count = sp_std::cmp::min(PairCount::<T>::get(&sell_asset_id, &buy_asset_id) - offset, count);

			let mut i = offset;
			while i < count {
				sellers.push(PairAccountList::<T>::get((&sell_asset_id, &buy_asset_id, i)).unwrap());
				i = i + 1;
			}

			sellers
		}
	}
}
