use mp_felt::Felt252Wrapper;
use mp_transactions::UserTransaction;
use parity_scale_codec::Encode;

pub type Call = Vec<Felt252Wrapper>;

/// Job Triggering Policy
/// Defines the conditions under which a job is triggered.
#[derive(Clone, Debug, PartialEq, Eq, parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct Policy {
    /// The block number at which the job starts to be valid.
    pub validity_start: u64,
    /// The block number at which the job ends to be valid.
    pub validity_end: u64,
}

/// User Policy
#[derive(Clone, Debug, PartialEq, Eq, parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct UserPolicy {
    /// Frequency of the job. (in blocks)
    pub frequency: u64,
}

/// Job Definition
/// Can only be built by the runtime.
#[derive(Clone, Debug, PartialEq, Eq, parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)]
pub struct Job {
    /// The block number at which the job is emitted.
    pub emission_block_number: u64,
    /// Maximum gas to be used for the job.
    pub max_gas: u64,
    /// Actual gas used for the job.
    pub actual_gas: u64,
    /// The calls to be executed when the job is triggered.
    pub calls: Vec<UserTransaction>,
    /// The according policy.
    pub policy: Policy,
}

/// User Job Definition
/// A user job is a job that is created by a user.
#[derive(Clone, Debug, PartialEq, Eq, parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct UserJob {
    /// The calls to be executed when the job is triggered.
    pub calls: Vec<Call>,
    /// The according policy.
    pub policy: UserPolicy,
}

pub const GAS_RATIO_SCALE: f64 = 100000.0;

impl Job {
    /// Compute the id of the job.
    /// Defines the priority of the job. (higher id means higher priority)
    pub fn compute_id(&self) -> u128 {
        let denom = if self.max_gas > 0 { self.max_gas } else { 1 };
        let gas_ratio = (self.actual_gas as f64 / denom as f64) * GAS_RATIO_SCALE;
        let encoded = self.encode();
        let call_hash: u128 = u128::from_le_bytes(sp_io::hashing::twox_128(&encoded));

        let id = gas_ratio as u128 + self.emission_block_number as u128 + call_hash;
        id
    }
}
