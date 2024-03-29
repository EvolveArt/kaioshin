use frame_support::pallet_prelude::Weight;
use frame_support::traits::OnIdle;
use frame_support::{assert_err, assert_ok};
use sp_core::Get;

use super::mock::default_mock::*;
use super::mock::*;
use crate::tests::get_dummy_user_job;
use crate::{Config, Error};

#[test]
fn given_no_jobs_are_registered_then_no_jobs_are_executed() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        assert_eq!(Autonomous::jobs().len(), 0);

        next_block(true);

        assert_eq!(Autonomous::jobs().len(), 0);
    })
}

#[test]
fn given_one_job_it_is_executed_on_time() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        let user_job = get_dummy_user_job();

        assert_ok!(Autonomous::register_job(none_origin, user_job));

        let jobs = Autonomous::jobs();
        let job = jobs.get(0).unwrap();
        assert_eq!(jobs.len(), 1);

        run_to_block(10, false);
        next_block(true);

        let is_executed = Autonomous::is_job_executed(job.0).unwrap();

        assert_eq!(is_executed, true);
    })
}

#[test]
fn given_one_job_it_is_removed_after_execution() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        let user_job = get_dummy_user_job();

        assert_ok!(Autonomous::register_job(none_origin, user_job));

        assert_eq!(Autonomous::jobs().len(), 1);

        run_to_block(11, false);

        next_block(true);

        assert_eq!(Autonomous::jobs().len(), 0);
    })
}

#[test]
fn given_one_job_it_is_not_executed_before_validity_start() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        let user_job = get_dummy_user_job();

        assert_ok!(Autonomous::register_job(none_origin, user_job));

        assert_eq!(Autonomous::jobs().len(), 1);

        run_to_block(9, false);

        next_block(true);

        assert_eq!(Autonomous::jobs().len(), 1);

        next_block(true);

        assert_eq!(Autonomous::jobs().len(), 0);
    })
}

#[test]
fn given_one_job_it_is_not_executed_after_validity_end() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        let user_job = get_dummy_user_job();

        assert_ok!(Autonomous::register_job(none_origin, user_job));

        assert_eq!(Autonomous::jobs().len(), 1);

        let validity_max_offset: u64 = <setup_mock::default_mock::MockRuntime as Config>::ValidityMaxOffset::get();

        run_to_block(12 + validity_max_offset, false);

        next_block(true);

        assert_eq!(Autonomous::jobs().len(), 1);
    })
}
