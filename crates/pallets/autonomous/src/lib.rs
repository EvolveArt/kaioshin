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
use sp_runtime::traits::UniqueSaturatedInto;

use crate::types::{Job, Policy, UserJob};

#[frame_support::pallet]
pub mod pallet {
    use frame_support::derive_impl;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use frame_system::RawOrigin;
    use mp_transactions::{InvokeTransaction, InvokeTransactionV1, UserTransaction};

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
    pub trait Config: frame_system::Config + pallet_starknet::Config {
        /// Maximum gas allowed for a job.
        #[pallet::constant]
        type MaxGas: Get<u64>;
        /// Maximum offset allowed for a job. (in blocks)
        #[pallet::constant]
        type ValidityMaxOffset: Get<u64>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn job_by_id)]
    pub(super) type Jobs<T: Config> = StorageMap<_, Identity, u128, Job, OptionQuery>;

    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn job_index_by_block_number)]
    pub(super) type JobIndex<T: Config> = StorageMap<_, Identity, u64, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn is_job_executed)]
    pub(super) type JobExecuted<T: Config> = StorageMap<_, Identity, u128, bool, OptionQuery>;

    /// The pallet custom errors.
    /// ERRORS
    #[pallet::error]
    pub enum Error<T> {
        JobAlreadyExecuted,
        InvalidJob,
        InvalidJobFrequency,
    }

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
                Ok(_) => {
                    log::info!("Job triggered successfully");
                    JobExecuted::<T>::insert(0, true);
                }
                Err(e) => {
                    log::error!("Error triggering job: {:?}", e);
                }
            }

            Weight::default()
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a job to be triggered.
        /// The job is stored in the storage and will be triggered when the conditions are met.
        ///
        /// # Arguments
        ///
        /// * `origin` - The origin of the call.
        /// * `job` - The job to be registered.
        #[pallet::call_index(0)]
        #[pallet::weight({0})]
        pub fn register_job(origin: OriginFor<T>, user_job: UserJob) -> DispatchResult {
            ensure_none(origin)?;

            let block_number =
                UniqueSaturatedInto::<u64>::unique_saturated_into(frame_system::Pallet::<T>::block_number());

            ensure!(user_job.policy.frequency >= 1, Error::<T>::InvalidJobFrequency);

            let index = JobIndex::<T>::get(block_number);
            let max_gas = T::MaxGas::get();

            // Estimate the gas required for the jobs.
            let user_transactions = user_job
                .calls
                .iter()
                .map(|calldata| {
                    let sequencer_address = pallet_starknet::Pallet::<T>::sequencer_address();
                    let nonce = pallet_starknet::Pallet::<T>::nonce(sequencer_address);

                    let transaction = InvokeTransaction::V1(InvokeTransactionV1 {
                        max_fee: 1e18 as u128,
                        signature: Default::default(),
                        nonce: nonce.into(),
                        sender_address: sequencer_address.into(),
                        calldata: calldata.clone(),
                        offset_version: false,
                    });

                    UserTransaction::Invoke(transaction)
                })
                .collect::<Vec<UserTransaction>>();

            let estimates = pallet_starknet::Pallet::<T>::estimate_fee(user_transactions.clone())?;
            let total_fee = estimates.iter().map(|x| x.0 + x.1).sum::<u64>();

            let policy = Policy {
                validity_start: block_number + user_job.policy.frequency - 1,
                validity_end: block_number + user_job.policy.frequency + T::ValidityMaxOffset::get(),
            };

            let job = Job {
                emission_block_number: block_number,
                index,
                max_gas,
                actual_gas: total_fee,
                calls: user_transactions,
                policy,
            };

            let job_id = job.compute_id();
            Jobs::<T>::insert(job_id, job);
            JobIndex::<T>::set(block_number, index + 1);

            Ok(())
        }
    }
}

/// Internal Functions
impl<T: Config> Pallet<T> {}
