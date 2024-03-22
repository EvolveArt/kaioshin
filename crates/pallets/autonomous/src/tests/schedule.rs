use frame_support::{assert_err, assert_ok};
use mp_felt::Felt252Wrapper;
use pallet_starknet::Error as StarknetError;

use super::mock::default_mock::*;
use super::mock::*;
use crate::tests::get_dummy_user_job;
use crate::types::{UserJob, UserPolicy};
use crate::{Config, Error};

#[test]
fn given_invalid_user_job_registration_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        let user_job = UserJob { calls: vec![], policy: UserPolicy { frequency: 10 } };
        assert_err!(Autonomous::register_job(none_origin.clone(), user_job), Error::<MockRuntime>::InvalidJob);

        let user_job = UserJob { calls: vec![vec![Felt252Wrapper::ZERO]], policy: UserPolicy { frequency: 0 } };
        assert_err!(Autonomous::register_job(none_origin.clone(), user_job), Error::<MockRuntime>::InvalidJobFrequency);

        let user_job = UserJob { calls: vec![vec![Felt252Wrapper::ZERO]], policy: UserPolicy { frequency: 10 } };
        assert_err!(
            Autonomous::register_job(none_origin, user_job),
            StarknetError::<MockRuntime>::TransactionExecutionFailed
        );
    })
}

#[test]
fn given_valid_user_job_registration_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        let user_job = get_dummy_user_job();

        assert_ok!(Autonomous::register_job(none_origin, user_job));
    })
}
