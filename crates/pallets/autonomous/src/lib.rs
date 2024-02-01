//! A Substrate pallet implementation for Autonomous Execution of Starknet Contracts.
//! This pallet is tightly coupled to the Starknet pallet.
//!
//! 1. Config: The trait Config is defined, which is used to configure the pallet by specifying the
//! parameters and types on which it depends. The trait also includes associated types for
//! RuntimeEvent, StateRoot, SystemHash, and TimestampProvider.
//!
//! 2. Hooks: The Hooks trait is implemented for the pallet, which includes methods to be executed
//! during the block lifecycle: on_finalize, on_initialize, on_runtime_upgrade, and offchain_worker.
//!
//! 3. Storage: Several storage items are defined, including Pending, CurrentBlock, BlockHash,
//! ContractClassHashes, ContractClasses, Nonces, StorageView, LastKnownEthBlock, and
//! FeeTokenAddress. These storage items are used to store and manage data related to the Starknet
//! pallet.
//!
//! 4. Genesis Configuration: The GenesisConfig struct is defined, which is used to set up the
//! initial state of the pallet during genesis. The struct includes fields for contracts,
//! contract_classes, storage, fee_token_address, and _phantom. A GenesisBuild implementation is
//! provided to build the initial state during genesis.
//!
//! 5. Events: A set of events are defined in the Event enum, including KeepStarknetStrange,
//! StarknetEvent, and FeeTokenAddressChanged. These events are emitted during the execution of
//! various pallet functions.
//!
//! 6.Errors: A set of custom errors are defined in the Error enum, which is used to represent
//! various error conditions during the execution of the pallet.
//!
//! 7. Dispatchable Functions: The Pallet struct implements several dispatchable functions (ping,
//! invoke, ...), which allow users to interact with the pallet and invoke state changes. These
//! functions are annotated with weight and return a DispatchResult.

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::large_enum_variant)]

/// Autonomous pallet.
/// Definition of the pallet's runtime storage items, events, errors, and dispatchable
/// functions.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
#[cfg(test)]
mod tests;

/// The pallet's runtime custom types.
pub mod types;

pub use pallet::*;

use crate::types::{Job, Policy};

#[frame_support::pallet]
pub mod pallet {
    use frame_support::derive_impl;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use frame_system::RawOrigin;
    use mp_transactions::{InvokeTransaction, InvokeTransactionV1};

    use super::*;

    /// Default preludes for [`Config`].
    pub mod config_preludes {
        use super::*;

        /// Default prelude sensible to be used in a testing environment.
        pub struct TestDefaultConfig;

        #[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig, no_aggregated_types)]
        impl frame_system::DefaultConfig for TestDefaultConfig {}
    }

    /// The pallet configuration trait
    #[pallet::config(with_default)]
    pub trait Config: frame_system::Config + pallet_starknet::Config {}

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn job_by_id)]
    pub(super) type Jobs<T: Config> = StorageMap<_, Identity, u128, Job, OptionQuery>;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// If we have some spare time left, the `on_idle` hook will trigger the jobs
        /// that need/can be triggered.
        /// Read the jobs that need to be triggered from the storage.
        /// Then it selects which ones to trigger based on the policy defined in the config.
        /// Finally we execute the jobs and update the storage accordingly.
        fn on_idle(_: BlockNumberFor<T>, _remaining_weight: Weight) -> Weight {
            let sequencer_address = pallet_starknet::Pallet::<T>::sequencer_address();
            let nonce = pallet_starknet::Pallet::<T>::nonce(sequencer_address);

            let transaction = InvokeTransaction::V1(InvokeTransactionV1 {
                max_fee: 1e18 as u128,
                signature: Default::default(),
                nonce: nonce.into(),
                sender_address: sequencer_address.into(),
                calldata: Vec::new(),
                offset_version: false,
            });

            match pallet_starknet::Pallet::<T>::invoke(RawOrigin::None.into(), transaction) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Error triggering job: {:?}", e);
                }
            }

            Weight::default()
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight({0})]
        pub fn set(origin: OriginFor<T>) -> DispatchResult {
            // This ensures that the function can only be called via unsigned transaction.
            ensure_none(origin)?;

            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {}
