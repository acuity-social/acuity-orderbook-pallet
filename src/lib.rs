#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;

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
#[cfg(feature = "std")]
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
#[derive(
    Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
pub struct ChainId([u8; 8]);

// 8 bytes chainId
// 2 bytes address type 0 - base, 1 - ERC20
// 2 bytes adapterId (smart contract)
// 20 bytes tokenAddress
#[derive(
    Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct AssetId(#[cfg_attr(feature = "std", serde(with = "serialize_array"))] [u8; 32]);

#[derive(
    Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
pub struct ForeignAccount([u8; 32]);

#[derive(
    Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PriceValue {
    pub price: u128,
    pub value: u128,
}

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::storage]
    #[pallet::getter(fn account_chain_id_account)]
    pub(super) type AccountForeignAccount<T: Config> =
        StorageDoubleMap<_, Identity, T::AccountId, Twox64Concat, ChainId, ForeignAccount>;

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
        /// The sell order is invalid.
        InvalidSellOrder,
        /// The sell order could not be found.
        SellOrderNotFound,
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(50_000_000)]
        pub fn set_foreign_account(
            origin: OriginFor<T>,
            chain_id: ChainId,
            foreign_account: ForeignAccount,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            <AccountForeignAccount<T>>::insert(&sender, chain_id, foreign_account);
            Self::deposit_event(Event::SetForeignAccount(sender, chain_id, foreign_account));
            Ok(().into())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(50_000_000)]
        pub fn set_order(
            origin: OriginFor<T>,
            sell_asset_id: AssetId,
            buy_asset_id: AssetId,
            price: u128,
            value: u128,
        ) -> DispatchResultWithPostInfo {
            let seller = ensure_signed(origin)?;
            let price_value = PriceValue { price, value };

            if price_value == Default::default() {
                return Err(Error::<T>::InvalidSellOrder.into());
            }

            //----------------------------------------

            Self::deposit_event(Event::SetOrder(sell_asset_id, buy_asset_id, price, value));
            Ok(().into())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(50_000_000)]
        pub fn remove_order(
            origin: OriginFor<T>,
            sell_asset_id: AssetId,
            buy_asset_id: AssetId,
        ) -> DispatchResultWithPostInfo {
            let seller = ensure_signed(origin)?;

            Ok(().into())
        }
        /*
                #[pallet::weight(50_000_000)]
                pub fn remove_orders_for_sell_asset(origin: OriginFor<T>, sell_asset_id: AssetId) -> DispatchResultWithPostInfo {
                    let sender = ensure_signed(origin)?;

                    Ok(().into())
                }

                #[pallet::weight(50_000_000)]
                pub fn remove_orders(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
                    let sender = ensure_signed(origin)?;

                    Ok(().into())
                }
        */
    }
}
