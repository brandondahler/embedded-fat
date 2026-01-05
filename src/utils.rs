macro_rules! ensure {
    ($value: expr, $error: expr) => {
        if !$value {
            return Err($error.into());
        }
    };
}

/// Propagates errors from results as an early-returning `Some` value.  Similar to `?`, but that
/// syntax doesn't work for `Option<Result<V, E>>`
macro_rules! propagate_iteration_error {
    ($result_expression: expr) => {
        match $result_expression {
            Result::Ok(value) => value,
            Result::Err(error) => return Some(Err(error.into())),
        }
    };
}

macro_rules! propagate_device_iteration_errors {
    ($result_expression: expr) => {
        propagate_iteration_error!(propagate_iteration_error!($result_expression))
    };
}

pub fn read_le_u16(data: &[u8], offset: usize) -> u16 {
    let mut bytes = [0; 2];
    bytes.copy_from_slice(&data[offset..offset + 2]);

    u16::from_le_bytes(bytes)
}

pub fn write_le_u16(data: &mut [u8], offset: usize, value: u16) {
    let bytes = value.to_le_bytes();

    data[offset..offset + 2].copy_from_slice(&bytes);
}

pub fn read_le_u32(data: &[u8], offset: usize) -> u32 {
    let mut bytes = [0; 4];
    bytes.copy_from_slice(&data[offset..offset + 4]);

    u32::from_le_bytes(bytes)
}

pub fn write_le_u32(data: &mut [u8], offset: usize, value: u32) {
    let bytes = value.to_le_bytes();

    data[offset..offset + 4].copy_from_slice(&bytes);
}

#[cfg(test)]
mod tests {
    use super::*;

    mod ensure {
        use super::*;

        #[test]
        fn value_false_propagates_error() {
            fn test() -> Result<(), MockError> {
                ensure!(false, MockError);

                Ok(())
            }

            assert!(test().is_err(), "Err should be returned");
        }

        #[test]
        fn value_false_converts_error() {
            fn test() -> Result<(), MockError> {
                ensure!(false, IntoMockError);

                Ok(())
            }

            assert!(test().is_err(), "Err should be returned");
        }

        #[test]
        fn value_true_does_nothing() {
            fn test() -> Result<(), MockError> {
                ensure!(true, MockError);

                Ok(())
            }

            assert!(test().is_ok(), "Ok should be returned");
        }
    }

    mod propagate_iteration_error {
        use super::*;

        #[test]
        fn error_propagated_as_some() {
            fn test() -> Option<Result<(), MockError>> {
                propagate_iteration_error!(Err(MockError));

                None
            }

            assert!(
                test().expect("Some should be returned").is_err(),
                "Err should be returned"
            );
        }

        #[test]
        fn error_converted_to_target() {
            fn test() -> Option<Result<(), MockError>> {
                Some(Ok(propagate_iteration_error!(Err(IntoMockError))))
            }

            assert!(
                test().expect("Some should be returned").is_err(),
                "Err should be returned"
            );
        }

        #[test]
        fn non_error_input_unwrapped() {
            fn test() -> Option<Result<(), MockError>> {
                let input: Result<(), MockError> = Ok(());

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
            fn test() -> Option<Result<(), MockError>> {
                let input: Result<Result<(), MockError>, MockError> = Ok(Err(MockError));

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
            fn test() -> Option<Result<(), MockError>> {
                let input: Result<Result<(), MockError>, MockError> = Err(MockError);

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
            fn test() -> Option<Result<(), MockError>> {
                let input: Result<Result<(), IntoMockError>, IntoMockError> =
                    Ok(Err(IntoMockError));

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
            fn test() -> Option<Result<(), MockError>> {
                let input: Result<Result<(), IntoMockError>, IntoMockError> = Err(IntoMockError);

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
            fn test() -> Option<Result<(), MockError>> {
                let input: Result<Result<(), IntoMockError>, IntoMockError> = Ok(Ok(()));

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

    struct MockError;
    struct IntoMockError;

    impl From<IntoMockError> for MockError {
        fn from(value: IntoMockError) -> Self {
            Self
        }
    }
}
