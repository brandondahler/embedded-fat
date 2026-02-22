use std::fmt::{Display, Formatter, UpperHex};

pub struct ArrayLiteral<'a, T: Display + UpperHex> {
    values: &'a Vec<T>,
    as_hex: bool,
}

impl<'a, T: Display + UpperHex> ArrayLiteral<'a, T> {
    pub fn new(values: &'a Vec<T>, as_hex: bool) -> Self {
        Self { values, as_hex }
    }
}

impl<T: ArrayLiteralElement> Display for ArrayLiteral<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[")?;

        for value in self.values {
            if self.as_hex {
                writeln!(f, "    0x{value:0width$X},", width = T::HEX_WIDTH)?;
            } else {
                writeln!(f, "    {value},")?;
            }
        }

        write!(f, "]")?;

        Ok(())
    }
}

pub trait ArrayLiteralElement: Display + UpperHex {
    const HEX_WIDTH: usize;
}

macro_rules! array_literal_element_impl {
    ($typ: ty, $hex_width: literal) => {
        impl ArrayLiteralElement for $typ {
            const HEX_WIDTH: usize = $hex_width;
        }
    };
}

array_literal_element_impl!(u8, 2);
array_literal_element_impl!(i8, 2);

array_literal_element_impl!(u16, 4);
array_literal_element_impl!(i16, 4);

array_literal_element_impl!(u32, 8);
array_literal_element_impl!(i32, 8);
