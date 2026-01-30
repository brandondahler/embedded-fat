use core::error::Error;
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum BiosParameterBlockError {
    AllocationTableCountInvalid,
    AllocationTableTooSmall,
    BytesPerSectorInvalid,
    FilesystemVersionUnsupported,
    FsInfoSectorNumberInvalid,
    MediaTypeInvalid,
    ReservedSectorCountInvalid,
    RootDirectoryEntryCountInvalid,
    RootDirectoryFileClusterNumberInvalid,
    SectorsPerClusterInvalid,
    SectorsPerAllocationTable16BitInvalid,
    SectorsPerAllocationTableNotSet,
    TotalSectorCount16BitInvalid,
    TotalSectorCountNotSet,
}

impl Error for BiosParameterBlockError {}

impl Display for BiosParameterBlockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            BiosParameterBlockError::AllocationTableCountInvalid => {
                write!(f, "BPB_NumFATs must not be zero")
            }
            BiosParameterBlockError::AllocationTableTooSmall => {
                write!(
                    f,
                    "The allocation table size defined in BPB_FATSz16 or BPB_FATSz32 isn't large enough to fit entries for all possible data clusters"
                )
            }
            BiosParameterBlockError::BytesPerSectorInvalid => {
                write!(f, "BPB_BytsPerSec must be one of the allowed values")
            }
            BiosParameterBlockError::FilesystemVersionUnsupported => {
                write!(f, "BPB_FSVer must be 0:0")
            }
            BiosParameterBlockError::FsInfoSectorNumberInvalid => {
                write!(f, "BPB_FSInfo must be greater than 0")
            }
            BiosParameterBlockError::MediaTypeInvalid => {
                write!(f, "BPB_Media must be one of the allowed values")
            }
            BiosParameterBlockError::ReservedSectorCountInvalid => {
                write!(f, "BPB_RsvdSecCnt must not be zero")
            }
            BiosParameterBlockError::RootDirectoryEntryCountInvalid => {
                write!(
                    f,
                    "BPB_RootEntCnt must be zero for FAT32 volumes and non-zero for FAT12 or FAT16 volumes"
                )
            }
            BiosParameterBlockError::RootDirectoryFileClusterNumberInvalid => {
                write!(f, "BPB_RootClus must be greater than 1")
            }
            BiosParameterBlockError::SectorsPerClusterInvalid => {
                write!(f, "BPB_SecPerClus must be a positive power of 2")
            }
            BiosParameterBlockError::SectorsPerAllocationTable16BitInvalid => {
                write!(
                    f,
                    "BPB_FATSz16 must be zero for FAT32 volumes and non-zero for FAT12 or FAT16 volumes"
                )
            }
            BiosParameterBlockError::SectorsPerAllocationTableNotSet => {
                write!(f, "Either BPB_FATSz16 or BPB_FATSz32 must be non-zero")
            }
            BiosParameterBlockError::TotalSectorCount16BitInvalid => {
                write!(f, "BPB_TotSec16 must be zero for FAT32 volumes")
            }
            BiosParameterBlockError::TotalSectorCountNotSet => {
                write!(f, "Either BPB_TotSec16 or BPB_TotSec32 must be non-zero")
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
