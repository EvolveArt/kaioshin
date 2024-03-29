//! A Substrate pallet implementation for Autonomous Execution of Starknet Contracts.
//! This pallet is tightly coupled to the Starknet pallet.
//!
/// You can find a thorough explanation of the design 
/// [here](https://github.com/keep-starknet-strange/madara/discussions/1309)

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

/// A maximum number of jobs. When number of jobs reaches this number, no new jobs may be
/// registered.
pub const MAX_JOBS: usize = 50;
pub const MINIMUM_GAS: u64 = 100_000;

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
    pub(super) type Jobs<T: Config> = StorageValue<_, Vec<(u128, Job)>, ValueQuery>;

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
        JobsLimitReached,
        JobAlreadyRegistered,
        JobGasLimitExceeded,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_idle(now: BlockNumberFor<T>, _remaining_weight: Weight) -> Weight {
            let mut all_jobs = Jobs::<T>::get();
            let block_number = UniqueSaturatedInto::<u64>::unique_saturated_into(now);

            all_jobs.iter_mut().for_each(|(job_id, job)| {
                // Skip if the job doesn't meet the timing constraints or has already been executed.
                if block_number < job.policy.validity_start
                    || block_number > job.policy.validity_end
                    || JobExecuted::<T>::get(*job_id).unwrap_or(false)
                {
                    return;
                }

                let transaction = job
                    .calls
                    .iter()
                    .find_map(|call| {
                        if let UserTransaction::Invoke(tx) = call {
                            Some(tx.clone()) // Clone here is necessary, but it's just one transaction rather than the whole jobs list.
                        } else {
                            None
                        }
                    })
                    .expect(
                        "Invalid transaction type; this should be unreachable if jobs are validated upon registration.",
                    );

                match pallet_starknet::Pallet::<T>::invoke(RawOrigin::None.into(), transaction) {
                    Ok(_) => {
                        log::info!("Job triggered successfully");
                        JobExecuted::<T>::insert(*job_id, true);
                    }
                    Err(e) => {
                        log::error!("Error triggering job: {:?}", e);
                    }
                }
            });

            // Remove executed jobs from the list to avoid processing them again.
            all_jobs.retain(|(job_id, _)| !JobExecuted::<T>::get(job_id).unwrap_or(false));
            Jobs::<T>::put(all_jobs);

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

            ensure!(!user_job.calls.is_empty(), Error::<T>::InvalidJob);
            ensure!(user_job.policy.frequency >= 1, Error::<T>::InvalidJobFrequency);

            let mut jobs = Jobs::<T>::get();
            ensure!(jobs.len() < MAX_JOBS, Error::<T>::JobsLimitReached);

            let block_number =
                UniqueSaturatedInto::<u64>::unique_saturated_into(frame_system::Pallet::<T>::block_number());

            let index = JobIndex::<T>::get(block_number) + 1;
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
                        signature: vec![], // no signature as the sequencer has a NoValidateAccount
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
            ensure!(total_fee <= max_gas, Error::<T>::JobGasLimitExceeded);

            let validity_start = block_number + user_job.policy.frequency - 1;
            let validity_end = block_number + user_job.policy.frequency + T::ValidityMaxOffset::get();

            let policy = Policy { validity_start, validity_end };

            let job = Job {
                emission_block_number: block_number,
                index,
                max_gas,
                actual_gas: total_fee,
                calls: user_transactions,
                policy,
            };

            let job_id = job.compute_id();
            // We don't want to add duplicate jobs, so we check whether the potential new
            // job is already present in the list. Because the list is always ordered, we can
            // leverage the binary search which makes this check O(log n).
            match jobs.iter().map(|(id, _)| *id).collect::<Vec<u128>>().binary_search(&job_id) {
                // If the search succeeds, the caller is already a member, so just return
                Ok(_) => Err(Error::<T>::JobAlreadyRegistered.into()),
                // If the search fails, the caller is not a member and we learned the index where
                // they should be inserted
                Err(job_index) => {
                    jobs.insert(job_index, (job_id, job.clone()));
                    Jobs::<T>::put(jobs);
                    JobIndex::<T>::set(block_number, index + 1);
                    JobExecuted::<T>::insert(job_id, false);

                    Ok(())
                }
            }
        }
    }
}

/// Internal Functions
impl<T: Config> Pallet<T> {
    pub fn jobs() -> Vec<(u128, Job)> {
        Jobs::<T>::get()
    }
}
