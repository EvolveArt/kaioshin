use mp_felt::Felt252Wrapper;

use crate::types::{UserJob, UserPolicy};

mod mock;

mod schedule;

mod on_idle;

pub fn get_dummy_user_job() -> UserJob {
    UserJob {
        calls: vec![vec![
            Felt252Wrapper::from_hex_be("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(), /* contract_address */
            Felt252Wrapper::from_hex_be("0x039a1491f76903a16feed0a6433bec78de4c73194944e1118e226820ad479701").unwrap(), /* selector for the `with_arg` external */
            Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(), /* calldata_len */
            Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000019").unwrap(), /* calldata[0] */
        ]],
        policy: UserPolicy { frequency: 10 },
    }
}

pub fn get_dummy_user_job_with_arg(arg: u128) -> UserJob {
    UserJob {
        calls: vec![vec![
            Felt252Wrapper::from_hex_be("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(), /* contract_address */
            Felt252Wrapper::from_hex_be("0x039a1491f76903a16feed0a6433bec78de4c73194944e1118e226820ad479701").unwrap(), /* selector for the `with_arg` external */
            Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(), /* calldata_len */
            Felt252Wrapper::from(arg), // calldata[0]
        ]],
        policy: UserPolicy { frequency: 10 },
    }
}
