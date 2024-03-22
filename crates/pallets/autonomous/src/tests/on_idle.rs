use frame_support::{assert_err, assert_ok};

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

        assert_eq!(Autonomous::jobs().len(), 1);

        next_block(true);

        let jobs = Autonomous::jobs();
        let job = jobs.get(0).unwrap();

        let is_executed = Autonomous::is_job_executed(job.0).unwrap();

        assert_eq!(is_executed, true);
    })
}
