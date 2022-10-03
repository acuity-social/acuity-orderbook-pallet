#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use sp_runtime::RuntimeDebug;
use codec::{Encode, Decode, MaxEncodedLen};
#[cfg(feature = "serde_derive")]
use serde::{Deserialize, Serialize};
use scale_info::TypeInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// Serialization shim for arbitrary arrays that is consistent with `polkadot-js`'s implementation.
///
/// `polkadot-js` sends us a `0x01020304`, but the default rust implementation for arrays expects a
/// `[0x01, 0x02, 0x03, 0x04]`. Here, we use a similar serialization as substrate uses for `vec`,
/// but we transform it to an array before returning.
#[cfg(feature = "serde_derive")]
pub mod serialize_array {
	use impl_serde::serialize::{deserialize_check_len, ExpectedLen};
	use serde::Deserializer;

	// default serialize is fine
	pub use impl_serde::serialize::serialize;

	pub use deserialize_array as deserialize;

	pub fn deserialize_array<'de, D, const T: usize>(deserializer: D) -> Result<[u8; T], D::Error>
	where
		D: Deserializer<'de>,
	{
		// All hail the stable const generics!
		let mut arr = [0u8; T];
		deserialize_check_len(deserializer, ExpectedLen::Exact(&mut arr[..]))?;

		Ok(arr)
	}
}


// 2 bytes chain type
// 6 bytes eth chainId
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ChainId([u8; 8]);

// 8 bytes chainId
// 2 bytes address type 0 - base, 1 - ERC20
// 2 bytes adapterId (smart contract)
// 20 bytes tokenAddress
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct AssetId(
    #[cfg_attr(feature = "std", serde(with = "serialize_array"))]
    [u8; 32]
);

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ForeignAccount([u8; 32]);

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
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
		NoOrder,
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
            let seller = ensure_signed(origin)?;
			let price_value = PriceValue{
				price: price,
				value: value,
			};

			if price_value == Default::default() {
				return Err(Error::<T>::NoOrder.into());
			}

            <AccountPairOrder<T>>::insert((&seller, sell_asset_id, buy_asset_id), price_value);

			// Check if this seller already has an order for this pair.
			if !<PairAccountIndex<T>>::contains_key((sell_asset_id, buy_asset_id, &seller)) {
				// Get the total number of sellers the pair already has.
				let count = PairCount::<T>::get(&sell_asset_id, &buy_asset_id);
				// Insert the new seller at the end of the list.
				<PairAccountList<T>>::insert((&sell_asset_id, &buy_asset_id, count), &seller);
				// Update the size of the list.
				<PairCount<T>>::insert(&sell_asset_id, &buy_asset_id, count + 1);
				// Store index + 1
				<PairAccountIndex<T>>::insert((sell_asset_id, buy_asset_id, seller), count + 1);
			}

            Self::deposit_event(Event::SetOrder(sell_asset_id, buy_asset_id, price, value));
			Ok(().into())
		}

		#[pallet::weight(50_000_000)]
		pub fn remove_order(origin: OriginFor<T>, sell_asset_id: AssetId, buy_asset_id: AssetId) -> DispatchResultWithPostInfo {
			let seller = ensure_signed(origin)?;

			// Get the index + 1 of the seller to be removed
			let i = match <PairAccountIndex<T>>::get((sell_asset_id, buy_asset_id, &seller)) {
                Some(i) => i,
                None => return Err(Error::<T>::NoOrder.into()),
            };

			//----------------------------------------

			// Delete the index from state.
			<PairAccountIndex<T>>::remove((sell_asset_id, buy_asset_id, &seller));
			// Get the list length.
			let count = PairCount::<T>::get(&sell_asset_id, &buy_asset_id);
			// Check if this is not the last account.
			if i != count {
				// Get the last account.
				let moving_account = <PairAccountList<T>>::get((&sell_asset_id, &buy_asset_id, count - 1)).unwrap();
				// Overwrite the seller being untrusted with the last account.
				<PairAccountList<T>>::insert((&sell_asset_id, &buy_asset_id, i - 1), &moving_account);
				// Update the index + 1 of the last account.
				<PairAccountIndex<T>>::insert((&sell_asset_id, &buy_asset_id, moving_account), i);
			}
			// Remove the last account.
			<PairAccountList<T>>::remove((&sell_asset_id, &buy_asset_id, count - 1));
			<PairCount<T>>::insert(&sell_asset_id, &buy_asset_id, count - 1);

			<AccountPairOrder<T>>::remove((seller, sell_asset_id, buy_asset_id));
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

		pub fn get_pair_sellers_orders(sell_asset_id: AssetId, buy_asset_id: AssetId, offset: u32, count: u32) -> (sp_std::prelude::Vec<T::AccountId>, sp_std::prelude::Vec<PriceValue>) {
			let sellers = Self::get_pair_sellers(sell_asset_id, buy_asset_id, offset, count);
			let mut orders = sp_std::prelude::Vec::new();

			for seller in &sellers {
				orders.push(<AccountPairOrder<T>>::get((seller, sell_asset_id, buy_asset_id)).unwrap());
			}

			(sellers, orders)
		}
	}
}
