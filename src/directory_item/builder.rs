use crate::directory_entry::{
    LONG_NAME_CHARACTERS_PER_ENTRY, LongNameDirectoryEntry, ShortNameDirectoryEntry,
};
use crate::directory_item::{DirectoryItem, DirectoryItemError};
use crate::encoding::Ucs2Character;
use crate::file_name::LONG_NAME_MAX_LENGTH;

const LONG_NAME_PADDING_CHARACTER: Ucs2Character = Ucs2Character::from_u16(0xFFFF).unwrap();
const LONG_NAME_MAX_ENTRY_COUNT: u8 =
    LONG_NAME_MAX_LENGTH.div_ceil(LONG_NAME_CHARACTERS_PER_ENTRY) as u8;

#[derive(Clone, Debug)]
pub struct DirectoryItemBuilder {
    current_entry_index: u8,

    long_name: [Ucs2Character; LONG_NAME_MAX_LENGTH],
    long_name_state: Option<LongNameState>,
}

#[derive(Clone, Debug)]
struct LongNameState {
    entry_count: u8,
    short_name_checksum: u8,
}

impl LongNameState {
    fn new(entry_count: u8, short_name_checksum: u8) -> LongNameState {
        Self {
            entry_count,
            short_name_checksum,
        }
    }
}

impl DirectoryItemBuilder {
    pub fn new() -> DirectoryItemBuilder {
        Self {
            current_entry_index: 0,

            long_name: [Ucs2Character::null(); LONG_NAME_MAX_LENGTH],
            long_name_state: None,
        }
    }

    pub fn add_long_name_entry(
        mut self,
        entry: LongNameDirectoryEntry,
    ) -> Result<Self, DirectoryItemError> {
        if self.current_entry_index == 0 {
            ensure!(
                entry.is_last_entry(),
                DirectoryItemError::LongNameFirstEntryInvalid
            );
        } else {
            ensure!(!entry.is_last_entry(), DirectoryItemError::LongNameOrphaned);
        }

        let long_name_state = self.long_name_state.get_or_insert_with(|| {
            LongNameState::new(entry.entry_number(), entry.short_name_checksum())
        });

        ensure!(
            entry.entry_number() == (long_name_state.entry_count - self.current_entry_index),
            DirectoryItemError::LongNameEntryNumberWrong
        );

        ensure!(
            entry.short_name_checksum() == long_name_state.short_name_checksum,
            DirectoryItemError::LongNameShortNameChecksumInconsistent
        );

        let long_name_offset = (entry.entry_number() - 1) as usize * LONG_NAME_CHARACTERS_PER_ENTRY;
        let mut null_encountered = false;

        for (character_index, character) in entry.ucs2_characters().iter().enumerate() {
            if *character == Ucs2Character::null() {
                ensure!(
                    self.current_entry_index == 0,
                    DirectoryItemError::LongNameCorrupted
                );
                ensure!(character_index != 0, DirectoryItemError::LongNameEmpty);

                null_encountered = true;
            } else if null_encountered {
                ensure!(
                    *character == LONG_NAME_PADDING_CHARACTER,
                    DirectoryItemError::LongNameCorrupted
                );

                continue;
            }

            let long_name_index = long_name_offset + character_index;

            // NOTE: value at long_name_index == LONG_NAME_MAX_LENGTH may validly be
            //   0x0000 or 0xFFFF, but both cases are handled above in their respective situations.
            ensure!(
                long_name_index < LONG_NAME_MAX_LENGTH,
                DirectoryItemError::LongNameTooLong
            );

            self.long_name[long_name_index] = *character;
        }

        self.current_entry_index += 1;

        Ok(self)
    }

    pub fn build(
        self,
        entry: ShortNameDirectoryEntry,
    ) -> Result<DirectoryItem, DirectoryItemError> {
        if let Some(long_name_state) = self.long_name_state {
            ensure!(
                self.current_entry_index == long_name_state.entry_count,
                DirectoryItemError::LongNameOrphaned
            );

            ensure!(
                entry.name().checksum() == long_name_state.short_name_checksum,
                DirectoryItemError::ShortNameChecksumMismatch
            );
        }

        Ok(DirectoryItem::new(entry, self.long_name.into()))
    }
}
