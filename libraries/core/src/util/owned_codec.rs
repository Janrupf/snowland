use skia_safe::{Codec, Data};
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct OwnedCodec {
    codec: Codec,
    _data: Vec<u8>,
}

impl OwnedCodec {
    pub fn new(data: Vec<u8>) -> Option<Self> {
        let owned_data = unsafe { Data::new_bytes(&data) };
        let codec = Codec::from_data(owned_data);

        codec.map(|codec| Self { codec, _data: data })
    }

    fn get_codec(&self) -> &Codec {
        &self.codec
    }

    fn get_codec_mut(&mut self) -> &mut Codec {
        &mut self.codec
    }
}

impl Deref for OwnedCodec {
    type Target = Codec;

    fn deref(&self) -> &Self::Target {
        self.get_codec()
    }
}

impl DerefMut for OwnedCodec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_codec_mut()
    }
}
