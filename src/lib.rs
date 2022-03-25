#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use sp_runtime::RuntimeDebug;
use frame_support::{
	traits::Currency,
};
use codec::{Encode, Decode, MaxEncodedLen};
use scale_info::TypeInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct AssetId([u8; 24]);

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Price(u128);

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use super::*;

	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;


	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The currency type that the charity deals in
        type Currency: Currency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn account_pair_order)]
	pub(super) type Orderbook<T: Config> = StorageNMap<
	    _,
	    (
	        NMapKey<Blake2_128Concat, T::AccountId>,	// seller
	        NMapKey<Blake2_128Concat, AssetId>,			// sell assetId
	        NMapKey<Blake2_128Concat, AssetId>,			// buy assetId
			NMapKey<Blake2_128Concat, Price>,			// selling price
	    ),
	    u128,
	    ValueQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
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
		pub fn set_order(origin: OriginFor<T>, sell_asset_id: AssetId, buy_asset_id: AssetId, price: Price, value: u128) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            <Orderbook<T>>::insert((sender, sell_asset_id, buy_asset_id, price), value);
//            Self::deposit_event(Event::AddToOrder(sender, chain_id, adapter_id, asset_id, price, foreign_address, value));
			Ok(().into())
		}

		#[pallet::weight(50_000_000)]
		pub fn remove_order(origin: OriginFor<T>, sell_asset_id: AssetId, buy_asset_id: AssetId, price: Price) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

			<Orderbook<T>>::remove((sender, sell_asset_id, buy_asset_id, price));
			Ok(().into())
		}

		#[pallet::weight(50_000_000)]
		pub fn remove_orders_for_pair(origin: OriginFor<T>, sell_asset_id: AssetId, buy_asset_id: AssetId) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			<Orderbook<T>>::remove_prefix((sender, sell_asset_id, buy_asset_id), None);
			Ok(().into())
		}

		#[pallet::weight(50_000_000)]
		pub fn remove_orders_for_sell_asset(origin: OriginFor<T>, sell_asset_id: AssetId) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			<Orderbook<T>>::remove_prefix((sender, sell_asset_id), None);
			Ok(().into())
		}
/*
		#[pallet::weight(50_000_000)]
		pub fn remove_orders(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			<Orderbook<T>>::remove_prefix(sender, None);
			Ok(().into())
		}
*/

/*		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}
*/
	}
}
