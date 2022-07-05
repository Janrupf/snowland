use std::io::Read;
use std::marker::PhantomData;

pub trait BitSerializable<const BYTES: usize> {
    fn into_buffer(self) -> [u8; BYTES];

    fn from_buffer(buffer: &[u8; BYTES]) -> Self;
}

#[derive(Debug)]
pub struct BitSerializableBuffer<T: BitSerializable<BYTES>, const BYTES: usize> {
    buffer: [u8; BYTES],
    current_length: usize,
    _data: PhantomData<T>,
}

impl<T: BitSerializable<BYTES>, const BYTES: usize> BitSerializableBuffer<T, BYTES> {
    pub fn new() -> Self {
        Self {
            buffer: [0; BYTES],
            current_length: 0,
            _data: PhantomData,
        }
    }

    pub fn append(&mut self, data: &[u8]) -> Vec<T> {
        let mut out = Vec::new();

        for b in data {
            self.buffer[self.current_length] = *b;
            self.current_length += 1;

            if self.current_length == BYTES {
                out.push(T::from_buffer(&self.buffer));
                self.current_length = 0;
            }
        }

        out
    }

    pub fn read_from<R: Read>(&mut self, mut reader: R) -> Result<Option<T>, std::io::Error> {
        let count = reader.read(&mut self.buffer[self.current_length..])?;
        self.current_length += count;

        if self.current_length == BYTES {
            let value = T::from_buffer(&self.buffer);
            self.current_length = 0;

            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    pub fn clear(&mut self) {
        self.current_length = 0;
    }
}

macro_rules! bit_serializable_number {
    ($number_type:ty) => {
        paste::paste! {
            #[allow(non_upper_case_globals)]
            const [<SIZE_OF_ $number_type>]: usize = ::std::mem::size_of::<$number_type>();

            pub type [<BitSerializable $number_type:camel Buffer>] =
                BitSerializableBuffer<$number_type, [<SIZE_OF_ $number_type>]>;
        }

        impl BitSerializable<{ ::std::mem::size_of::<$number_type>() }> for $number_type {
            fn into_buffer(self) -> [u8; paste::paste! { [<SIZE_OF_ $number_type>] }] {
                let mut buffer = [0; paste::paste! { [<SIZE_OF_ $number_type>] }];

                for i in 0..paste::paste! { [<SIZE_OF_ $number_type>] } {
                    buffer[i] = (self >> (8 * i)) as u8;
                }

                buffer
            }

            fn from_buffer(buffer: &[u8; paste::paste! { [<SIZE_OF_ $number_type>] }]) -> Self {
                let mut out = 0;

                for i in 0..paste::paste! { [<SIZE_OF_ $number_type>] } {
                    out |= (buffer[i] as $number_type) << (8 * i);
                }

                out
            }
        }
    };
    ($($number_type:ty),*) => {
        $(bit_serializable_number!($number_type);)*
    }
}

bit_serializable_number!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn serialization_preserves_value() {
        let input = usize::MAX / 2;

        let serialized = input.into_buffer();
        let deserialized = usize::from_buffer(&serialized);

        assert_eq!(input, deserialized);
    }

    #[test]
    pub fn buffer_preserves_value() {
        let input = usize::MAX / 4;

        let serialized = input.into_buffer();

        let mut buffer = BitSerializableUsizeBuffer::new();
        assert_eq!(buffer.append(&serialized[..3]), Vec::new());
        assert_eq!(buffer.append(&serialized[3..]), vec![input]);
    }
}
