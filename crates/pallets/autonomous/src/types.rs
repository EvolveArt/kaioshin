use mp_felt::Felt252Wrapper;

pub type Call = Vec<Felt252Wrapper>;

/// Job Triggering Policy
/// Defines the conditions under which a job is triggered.
#[derive(Clone, Debug, PartialEq, Eq, parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct Policy {
    frequency: u64,
}

/// Job Definition
#[derive(Clone, Debug, PartialEq, Eq, parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct Job {
    /// The block number of the last time the job was executed.
    last_block_executed: u64,
    /// The calls to be executed when the job is triggered.
    calls: Vec<Call>,
    /// The according policy.
    policy: Policy,
}
