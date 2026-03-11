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

// NOTE: The tests need the macros so this mod declaration must be after them
#[cfg(test)]
mod tests;

pub fn read_le_u16(bytes: &[u8], offset: usize) -> u16 {
    let mut value_bytes = [0; 2];
    value_bytes.copy_from_slice(&bytes[offset..offset + 2]);

    u16::from_le_bytes(value_bytes)
}

pub fn write_le_u16(bytes: &mut [u8], offset: usize, value: u16) {
    let value_bytes = value.to_le_bytes();

    bytes[offset..offset + 2].copy_from_slice(&value_bytes);
}

pub fn read_le_u32(data: &[u8], offset: usize) -> u32 {
    let mut value_bytes = [0; 4];
    value_bytes.copy_from_slice(&data[offset..offset + 4]);

    u32::from_le_bytes(value_bytes)
}

pub fn write_le_u32(bytes: &mut [u8], offset: usize, value: u32) {
    let value_bytes = value.to_le_bytes();

    bytes[offset..offset + 4].copy_from_slice(&value_bytes);
}
