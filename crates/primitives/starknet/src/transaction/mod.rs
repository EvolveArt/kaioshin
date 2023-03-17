//! Starknet transaction related functionality.

use alloc::vec;

use blockifier::block_context::BlockContext;
use blockifier::state::cached_state::TransactionalState;
use blockifier::state::state_api::{State, StateReader};
use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::transactions::ExecutableTransaction;
use blockifier::transaction::objects::{TransactionExecutionInfo, TransactionExecutionResult};

use frame_support::BoundedVec;
use sp_core::{ConstU32, H256, U256};
use starknet_api::api_core::{ContractAddress as StarknetContractAddress, Nonce};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Fee, InvokeTransaction, TransactionHash, TransactionSignature, TransactionVersion};

use crate::block::wrapper::block::Block;
use crate::block::serialize::SerializeBlockContext;
use crate::execution::{ContractAddress, CallEntryPoint};

type MaxArraySize = ConstU32<4294967295>;

/// Representation of a Starknet event.
#[derive(Clone, Debug, PartialEq, Eq, codec::Encode, codec::Decode, scale_info::TypeInfo, codec::MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct Event {
    /// The keys (topics) of the event.
    pub keys: BoundedVec<H256, MaxArraySize>,
    /// The data of the event.
    pub data: BoundedVec<H256, MaxArraySize>,
    /// The address that emited the event
    pub from_address: H256,
}

impl Event {
    /// Creates a new instance of an event.
    pub fn new(keys: BoundedVec<H256, MaxArraySize>, data: BoundedVec<H256, MaxArraySize>, from_address: H256) -> Self {
        Self { keys, data, from_address }
    }

    /// Creates an empty event.
    pub fn empty() -> Self {
        Self {
            keys: BoundedVec::try_from(vec![]).unwrap(),
            data: BoundedVec::try_from(vec![]).unwrap(),
            from_address: H256::zero(),
        }
    }
}
impl Default for Event {
    fn default() -> Self {
        let one = H256::from_slice(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
        ]);
        Self {
            keys: BoundedVec::try_from(vec![one, one]).unwrap(),
            data: BoundedVec::try_from(vec![one, one]).unwrap(),
            from_address: one,
        }
    }
}

/// Representation of a Starknet transaction.
#[derive(Clone, Debug, PartialEq, Eq, codec::Encode, codec::Decode, scale_info::TypeInfo, codec::MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct Transaction {
    /// The version of the transaction.
    pub version: U256,
    /// Transaction hash.
    pub hash: H256,
    /// Signature.
    pub signature: BoundedVec<H256, MaxArraySize>,
    /// Events.
    pub events: BoundedVec<Event, MaxArraySize>,
    /// Sender Address
    pub sender_address: ContractAddress,
    /// Nonce
    pub nonce: U256,
	/// Call entrypoint
	pub call_entrypoint: CallEntryPoint
}

impl Transaction {
    /// Creates a new instance of a transaction.
    pub fn new(
        version: U256,
        hash: H256,
        signature: BoundedVec<H256, MaxArraySize>,
        events: BoundedVec<Event, MaxArraySize>,
        sender_address: ContractAddress,
        nonce: U256,
		call_entrypoint: CallEntryPoint
    ) -> Self {
        Self { version, hash, signature, events, sender_address, nonce, call_entrypoint }
    }

    /// Creates a new instance of a transaction without signature.
    pub fn from_tx_hash(hash: H256) -> Self {
        Self { hash, ..Self::default() }
    }

    /// Converts a transaction to a blockifier transaction
    ///
    /// # Arguments
    ///
    /// * `self` - The transaction to convert
    ///
    /// # Returns
    ///
    /// * `AccountTransaction` - The converted transaction
    pub fn to_invoke_tx(self: &Self) -> AccountTransaction {
        AccountTransaction::Invoke(InvokeTransaction {
            transaction_hash: TransactionHash(StarkFelt::new(self.hash.0).unwrap()),
            max_fee: Fee(2),
            version: TransactionVersion(StarkFelt::new(self.version.into()).unwrap()),
            signature: TransactionSignature(
                self.signature.clone().into_inner().iter().map(|x| StarkFelt::new(x.0).unwrap()).collect(),
            ),
            nonce: Nonce(StarkFelt::new(self.nonce.into()).unwrap()),
            sender_address: StarknetContractAddress::try_from(StarkFelt::new(self.sender_address).unwrap()).unwrap(),
			calldata: self.call_entrypoint.to_starknet_call_entry_point().calldata,
			entry_point_selector: Some(self.call_entrypoint.to_starknet_call_entry_point().entry_point_selector),
        })
    }

	/// Executes a transaction
	///
	/// # Arguments
	///
	/// * `self` - The transaction to execute
	/// * `state` - The state to execute the transaction on
	/// * `block` - The block to execute the transaction on
	///
	/// # Returns
	///
	/// * `TransactionExecutionResult<TransactionExecutionInfo>` - The result of the transaction execution
	pub fn execute<S: StateReader>(self: &Self, state: &mut TransactionalState<'_, S>, block: Block) -> TransactionExecutionResult<TransactionExecutionInfo> {
		let tx = self.to_invoke_tx();
		let block_context = BlockContext::serialize(block.header);
		let result = tx.execute_raw(state, &block_context);
		result
	}
}

impl Default for Transaction {
    fn default() -> Self {
        let one = H256::from_slice(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
        ]);
        Self {
            version: U256::default(),
            hash: one,
            signature: BoundedVec::try_from(vec![one, one]).unwrap(),
            events: BoundedVec::try_from(vec![Event::default(), Event::default()]).unwrap(),
            nonce: U256::default(),
            sender_address: ContractAddress::default(),
			call_entrypoint: CallEntryPoint::default()
        }
    }
}
