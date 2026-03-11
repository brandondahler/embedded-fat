use super::*;
use crate::mock::{CoreError, IntoCoreError};

mod ensure {
    use super::*;

    #[test]
    fn value_false_propagates_error() {
        fn test() -> Result<(), CoreError> {
            ensure!(false, CoreError);

            Ok(())
        }

        assert!(test().is_err(), "Err should be returned");
    }

    #[test]
    fn value_false_converts_error() {
        fn test() -> Result<(), CoreError> {
            ensure!(false, IntoCoreError);

            Ok(())
        }

        assert!(test().is_err(), "Err should be returned");
    }

    #[test]
    fn value_true_does_nothing() {
        fn test() -> Result<(), CoreError> {
            ensure!(true, CoreError);

            Ok(())
        }

        assert!(test().is_ok(), "Ok should be returned");
    }
}

mod propagate_iteration_error {
    use super::*;

    #[test]
    fn error_propagated_as_some() {
        fn test() -> Option<Result<(), CoreError>> {
            propagate_iteration_error!(Err(CoreError));

            None
        }

        assert!(
            test().expect("Some should be returned").is_err(),
            "Err should be returned"
        );
    }

    #[test]
    fn error_converted_to_target() {
        fn test() -> Option<Result<(), CoreError>> {
            Some(Ok(propagate_iteration_error!(Err(IntoCoreError))))
        }

        assert!(
            test().expect("Some should be returned").is_err(),
            "Err should be returned"
        );
    }

    #[test]
    fn non_error_input_unwrapped() {
        fn test() -> Option<Result<(), CoreError>> {
            let input: Result<(), CoreError> = Ok(());

            Some(Ok(propagate_iteration_error!(input)))
        }

        assert!(
            test().expect("Some should be returned").is_ok(),
            "Ok should be returned"
        );
    }
}

mod propagate_device_iteration_errors {
    use super::*;

    #[test]
    fn inner_error_propagated_as_some() {
        fn test() -> Option<Result<(), CoreError>> {
            let input: Result<Result<(), CoreError>, CoreError> = Ok(Err(CoreError));

            propagate_device_iteration_errors!(input);

            None
        }

        assert!(
            test().expect("Some should be returned").is_err(),
            "Err should be returned"
        );
    }

    #[test]
    fn outer_error_propagated_as_some() {
        fn test() -> Option<Result<(), CoreError>> {
            let input: Result<Result<(), CoreError>, CoreError> = Err(CoreError);

            propagate_device_iteration_errors!(input);

            None
        }

        assert!(
            test().expect("Some should be returned").is_err(),
            "Err should be returned"
        );
    }

    #[test]
    fn inner_error_converted_to_target() {
        fn test() -> Option<Result<(), CoreError>> {
            let input: Result<Result<(), IntoCoreError>, IntoCoreError> = Ok(Err(IntoCoreError));

            propagate_device_iteration_errors!(input);

            None
        }

        assert!(
            test().expect("Some should be returned").is_err(),
            "Err should be returned"
        );
    }

    #[test]
    fn outer_error_converted_to_target() {
        fn test() -> Option<Result<(), CoreError>> {
            let input: Result<Result<(), IntoCoreError>, IntoCoreError> = Err(IntoCoreError);

            propagate_device_iteration_errors!(input);

            None
        }

        assert!(
            test().expect("Some should be returned").is_err(),
            "Err should be returned"
        );
    }

    #[test]
    fn non_error_input_unwrapped() {
        fn test() -> Option<Result<(), CoreError>> {
            let input: Result<Result<(), IntoCoreError>, IntoCoreError> = Ok(Ok(()));

            Some(Ok(propagate_device_iteration_errors!(input)))
        }

        assert!(
            test().expect("Some should be returned").is_ok(),
            "Ok should be returned"
        );
    }
}

mod read_le_u16 {
    use super::*;

    #[test]
    fn exact_size_input_read_correctly() {
        let input = [0x12, 0x34];

        assert_eq!(
            read_le_u16(&input, 0),
            0x3412,
            "Correct value should be returned"
        );
    }

    #[test]
    fn oversized_input_read_correctly() {
        let input = [0x12, 0x34, 0x56, 0x78];

        assert_eq!(
            read_le_u16(&input, 0),
            0x3412,
            "Correct value should be returned"
        );
    }

    #[test]
    fn offset_input_read_correctly() {
        let input = [0x12, 0x34, 0x56];

        assert_eq!(
            read_le_u16(&input, 1),
            0x5634,
            "Correct value should be returned"
        );
    }
}

mod write_le_u16 {
    use super::*;

    #[test]
    fn exact_size_output_written_correctly() {
        let mut output = [0xFF; 2];

        write_le_u16(&mut output, 0, 0x3412);

        assert_eq!(output, [0x12, 0x34], "Correct value should be written");
    }

    #[test]
    fn oversized_output_written_correctly() {
        let mut output = [0xFF; 4];

        write_le_u16(&mut output, 0, 0x3412);

        assert_eq!(
            output,
            [0x12, 0x34, 0xFF, 0xFF],
            "Correct value should be written"
        );
    }

    #[test]
    fn offset_output_written_correctly() {
        let mut output = [0xFF; 3];

        write_le_u16(&mut output, 1, 0x3412);

        assert_eq!(
            output,
            [0xFF, 0x12, 0x34],
            "Correct value should be written"
        );
    }
}

mod read_le_u32 {
    use super::*;

    #[test]
    fn exact_size_input_read_correctly() {
        let input = [0x12, 0x34, 0x56, 0x78];

        assert_eq!(
            read_le_u32(&input, 0),
            0x78563412,
            "Correct value should be returned"
        );
    }

    #[test]
    fn oversized_input_read_correctly() {
        let input = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF1];

        assert_eq!(
            read_le_u32(&input, 0),
            0x78563412,
            "Correct value should be returned"
        );
    }

    #[test]
    fn offset_input_read_correctly() {
        let input = [0x12, 0x34, 0x56, 0x78, 0x9A];

        assert_eq!(
            read_le_u32(&input, 1),
            0x9A785634,
            "Correct value should be returned"
        );
    }
}

mod write_le_u32 {
    use super::*;

    #[test]
    fn exact_size_output_written_correctly() {
        let mut output = [0xFF; 4];

        write_le_u32(&mut output, 0, 0x78563412);

        assert_eq!(
            output,
            [0x12, 0x34, 0x56, 0x78],
            "Correct value should be written"
        );
    }

    #[test]
    fn oversized_output_written_correctly() {
        let mut output = [0xFF; 8];

        write_le_u32(&mut output, 0, 0x78563412);

        assert_eq!(
            output,
            [0x12, 0x34, 0x56, 0x78, 0xFF, 0xFF, 0xFF, 0xFF],
            "Correct value should be written"
        );
    }

    #[test]
    fn offset_output_written_correctly() {
        let mut output = [0xFF; 5];

        write_le_u32(&mut output, 1, 0x78563412);

        assert_eq!(
            output,
            [0xFF, 0x12, 0x34, 0x56, 0x78],
            "Correct value should be written"
        );
    }
}
