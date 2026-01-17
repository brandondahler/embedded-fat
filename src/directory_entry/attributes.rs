use bitflags::bitflags;

bitflags! {
    /// Represents a set of flags.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct DirectoryEntryAttributes: u8 {
        const ReadOnly      = 1 << 0;
        const Hidden        = 1 << 1;
        const System        = 1 << 2;
        const VolumeLabel   = 1 << 3;
        const Subdirectory  = 1 << 4;
        const Archive       = 1 << 5;

        const LongName = Self::ReadOnly.bits() | Self::Hidden.bits() | Self::System.bits() | Self::VolumeLabel.bits();
    }
}
