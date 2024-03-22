use mp_felt::Felt252Wrapper;

use crate::types::{UserJob, UserPolicy};

mod mock;

mod schedule;

mod on_idle;

pub fn get_dummy_user_job() -> UserJob {
    UserJob {
        calls: vec![vec![
            Felt252Wrapper::from_hex_be("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(), /* contract_address */
            Felt252Wrapper::from_hex_be("0x00e7def693d16806ca2a2f398d8de5951344663ba77f340ed7a958da731872fc").unwrap(), /* selector for the `with_arg` external */
            Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(), /* calldata_len */
            Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000019").unwrap(), /* calldata[0] */
        ]],
        policy: UserPolicy { frequency: 10 },
    }
}
