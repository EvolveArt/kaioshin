use frame_support::{assert_err, assert_ok};
use mp_felt::Felt252Wrapper;
use pallet_starknet::Error as StarknetError;
use sp_core::Get;

use super::mock::default_mock::*;
use super::mock::*;
use crate::tests::{get_dummy_user_job, get_dummy_user_job_with_arg};
use crate::types::{UserJob, UserPolicy};
use crate::{Config, Error, MAX_JOBS};

#[test]
fn given_invalid_user_job_registration_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let signed_origin = RuntimeOrigin::signed(1);

        let user_job = UserJob { calls: vec![], policy: UserPolicy { frequency: 10 } };
        assert_err!(Autonomous::register_job(signed_origin.clone(), user_job), Error::<MockRuntime>::InvalidJob);

        let user_job = UserJob { calls: vec![vec![Felt252Wrapper::ZERO]], policy: UserPolicy { frequency: 0 } };
        assert_err!(
            Autonomous::register_job(signed_origin.clone(), user_job),
            Error::<MockRuntime>::InvalidJobFrequency
        );

        let user_job = UserJob { calls: vec![vec![Felt252Wrapper::ZERO]], policy: UserPolicy { frequency: 10 } };
        assert_err!(
            Autonomous::register_job(signed_origin, user_job),
            StarknetError::<MockRuntime>::TransactionExecutionFailed
        );
    })
}

#[test]
fn given_valid_user_job_registration_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let signed_origin = RuntimeOrigin::signed(1);

        let user_job = get_dummy_user_job();

        assert_ok!(Autonomous::register_job(signed_origin, user_job));

        let all_jobs = Autonomous::jobs();
        let (job_id, job) = all_jobs.get(0).unwrap();

        assert_eq!(job.emission_block_number, 2);

        let max_gas: u64 = <setup_mock::default_mock::MockRuntime as Config>::MaxGas::get();
        assert_eq!(job.max_gas, max_gas);

        let validity_max_offset: u64 = <setup_mock::default_mock::MockRuntime as Config>::ValidityMaxOffset::get();
        assert_eq!(job.policy.validity_start, 11);
        assert_eq!(job.policy.validity_end, 12 + validity_max_offset);

        assert!(job.actual_gas > 0);
        assert_eq!(job.calls.len(), 1);

        let is_executed = Autonomous::is_job_executed(job_id).unwrap();
        assert_eq!(is_executed, false);

        let job_index = Autonomous::job_index_by_block_number(2);
        assert_eq!(job_index, 2);
    })
}

#[test]
fn fail_to_register_more_than_max_jobs() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let signed_origin = RuntimeOrigin::signed(1);

        for i in 0..MAX_JOBS {
            let user_job = get_dummy_user_job_with_arg(i.try_into().unwrap());
            assert_ok!(Autonomous::register_job(signed_origin.clone(), user_job.clone()));
        }

        let user_job = get_dummy_user_job_with_arg(MAX_JOBS.try_into().unwrap());
        assert_err!(Autonomous::register_job(signed_origin, user_job), Error::<MockRuntime>::JobsLimitReached);
    })
}

#[test]
fn fail_to_register_job_twice() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let signed_origin = RuntimeOrigin::signed(1);

        let user_job = get_dummy_user_job();

        assert_ok!(Autonomous::register_job(signed_origin.clone(), user_job.clone()));
        assert_err!(Autonomous::register_job(signed_origin, user_job), Error::<MockRuntime>::JobAlreadyRegistered);
    })
}
