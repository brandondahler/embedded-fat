mod attributes;
mod error;
mod free;
mod iterator;
mod long_name;
mod short_name;

pub use attributes::*;
pub use error::*;
pub use free::*;
pub use iterator::*;
pub use long_name::*;
pub use short_name::*;

#[cfg(feature = "sync")]
use embedded_io::{Seek, Write};

#[cfg(feature = "async")]
use embedded_io_async::{Seek as AsyncSeek, Write as AsyncWrite};

pub const DIRECTORY_ENTRY_SIZE: usize = 32;

#[derive(Clone, Debug)]
pub enum DirectoryEntry {
    Free(FreeDirectoryEntry),
    ShortName(ShortNameDirectoryEntry),
    LongName(LongNameDirectoryEntry),
}

impl DirectoryEntry {
    pub fn from_bytes(
        entry_bytes: &[u8; DIRECTORY_ENTRY_SIZE],
    ) -> Result<DirectoryEntry, DirectoryEntryError> {
        if matches!(entry_bytes[0], 0x00) {
            Ok(FreeDirectoryEntry::AllFollowing.into())
        } else if matches!(entry_bytes[0], 0xE5) {
            Ok(FreeDirectoryEntry::CurrentOnly.into())
        } else if entry_bytes[11] & 0x0F > 0 {
            Ok(LongNameDirectoryEntry::from_bytes(entry_bytes)?.into())
        } else {
            Ok(ShortNameDirectoryEntry::from_bytes(entry_bytes)?.into())
        }
    }
}

impl From<FreeDirectoryEntry> for DirectoryEntry {
    fn from(value: FreeDirectoryEntry) -> Self {
        Self::Free(value)
    }
}

impl From<LongNameDirectoryEntry> for DirectoryEntry {
    fn from(value: LongNameDirectoryEntry) -> Self {
        Self::LongName(value)
    }
}

impl From<ShortNameDirectoryEntry> for DirectoryEntry {
    fn from(value: ShortNameDirectoryEntry) -> Self {
        Self::ShortName(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AsciiOnlyEncoder;
    use crate::encoding::Ucs2Character;
    use crate::file_name::ShortFileName;

    mod from_bytes {
        use super::*;

        #[test]
        fn free_all_following_parsed_correctly() {
            let mut data = [0x00; DIRECTORY_ENTRY_SIZE];
            data[0] = 0x00;

            let entry = DirectoryEntry::from_bytes(&data).expect("Ok should be returned");

            assert!(
                matches!(
                    entry,
                    DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
                ),
                "AllFollowing free entry should be returned"
            );
        }

        #[test]
        fn free_current_only_parsed_correctly() {
            let mut data = [0x00; DIRECTORY_ENTRY_SIZE];
            data[0] = 0xE5;

            let entry = DirectoryEntry::from_bytes(&data).expect("Ok should be returned");

            assert!(
                matches!(entry, DirectoryEntry::Free(FreeDirectoryEntry::CurrentOnly)),
                "CurrentOnly free entry should be returned"
            );
        }

        #[test]
        fn short_name_parsed_correctly() {
            let short_name_entry = ShortNameDirectoryEntry::builder()
                .name(ShortFileName::from_str(&AsciiOnlyEncoder, "A").unwrap())
                .attributes(DirectoryEntryAttributes::empty())
                .first_cluster_number(2)
                .file_size(0)
                .build();

            let mut data = [0x00; DIRECTORY_ENTRY_SIZE];
            short_name_entry.write(&mut data);

            let entry = DirectoryEntry::from_bytes(&data).expect("Ok should be returned");

            assert!(
                matches!(entry, DirectoryEntry::ShortName(_)),
                "ShortName entry should be returned"
            );
        }

        #[test]
        fn short_name_error_propagated() {
            let mut data = [0x00; DIRECTORY_ENTRY_SIZE];
            data[0] = 0x01;

            let error = DirectoryEntry::from_bytes(&data).expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryError::ShortNameEntryInvalid(_)),
                "ShortNameEntryInvalid should be returned"
            );
        }

        #[test]
        fn long_name_parsed_correctly() {
            let mut ucs2_characters =
                [Ucs2Character::from_u16(0xFFFF).unwrap(); LONG_NAME_CHARACTERS_PER_ENTRY];
            ucs2_characters[0] = Ucs2Character::from_char('A').unwrap();
            ucs2_characters[1] = Ucs2Character::null();

            let long_name_entry = LongNameDirectoryEntry::builder()
                .ucs2_characters(ucs2_characters)
                .order_byte(0x01)
                .short_name_checksum(0x00)
                .build();

            let mut data = [0x00; DIRECTORY_ENTRY_SIZE];
            long_name_entry.write(&mut data);

            let entry = DirectoryEntry::from_bytes(&data).expect("Ok should be returned");

            assert!(
                matches!(entry, DirectoryEntry::LongName(_)),
                "LongName entry should be returned"
            );
        }

        #[test]
        fn long_name_error_propagated() {
            let mut data = [0x00; DIRECTORY_ENTRY_SIZE];
            data[0] = 0x3F;
            data[11] = 0x0F;

            let error = DirectoryEntry::from_bytes(&data).expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryError::LongNameEntryInvalid(_)),
                "LongNameEntryInvalid should be returned"
            );
        }
    }
}
