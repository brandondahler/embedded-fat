#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct UnicodePlane {
    value: u8,
}

impl UnicodePlane {
    pub fn new(value: u8) -> UnicodePlane {
        assert!(value <= 16);

        UnicodePlane { value }
    }

    pub fn for_codepoint(codepoint: u32) -> Self {
        let plane_value = codepoint >> 16;

        Self::new(u8::try_from(plane_value).expect("plane value too large"))
    }

    pub fn value(&self) -> u8 {
        self.value
    }
}
