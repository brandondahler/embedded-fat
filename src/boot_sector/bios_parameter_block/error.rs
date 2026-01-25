use core::error::Error;
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum BiosParameterBlockError {
    InvalidBytesPerSector,
    InvalidSectorsPerCluster,
    InvalidReservedSectorCount,
    InvalidFatCount,
    InvalidRootDirectoryEntryCount,
    InvalidMediaType,
    InvalidTotalSectorCount16Bit,
    InvalidFatSectorCount16Bit,
    InvalidTotalSectorCount32Bit,
}

impl Error for BiosParameterBlockError {}

impl Display for BiosParameterBlockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            BiosParameterBlockError::InvalidBytesPerSector => write!(f, "Invalid bytes per sector"),
            BiosParameterBlockError::InvalidSectorsPerCluster => {
                write!(f, "Invalid Sectors per cluster")
            }
            BiosParameterBlockError::InvalidReservedSectorCount => {
                write!(f, "Invalid reserved sector count")
            }
            BiosParameterBlockError::InvalidFatCount => write!(f, "Invalid FAT count"),
            BiosParameterBlockError::InvalidRootDirectoryEntryCount => {
                write!(f, "Invalid root directory entry count")
            }
            BiosParameterBlockError::InvalidMediaType => write!(f, "Invalid media type"),
            BiosParameterBlockError::InvalidTotalSectorCount16Bit => {
                write!(f, "Invalid total sector count16")
            }
            BiosParameterBlockError::InvalidFatSectorCount16Bit => {
                write!(f, "Invalid total sector count16")
            }
            BiosParameterBlockError::InvalidTotalSectorCount32Bit => {
                write!(f, "Invalid total sector count32")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use strum::IntoEnumIterator;

    mod display {
        use super::*;

        #[test]
        fn produces_non_empty_value() {
            for value in BiosParameterBlockError::iter() {
                assert!(
                    !value.to_string().is_empty(),
                    "Display implementation should be non-empty"
                );
            }
        }
    }
}
