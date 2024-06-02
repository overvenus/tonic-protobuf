//! A [`tonic::Codec`](https://docs.rs/tonic/0.11.0/tonic/codec/trait.Codec.html)
//! that implements `application/grpc+proto` via the rust-protobuf.

#[cfg(feature = "protobuf-v3")]
mod protobuf_v3 {
    use std::marker::PhantomData;

    use bytes::{Buf, BufMut};
    use protobuf::Message;
    use tonic::{
        codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder},
        Code, Status,
    };

    /// A [`Codec`] that implements `application/grpc+proto` via the [rust-protobuf v3](https://crates.io/crates/protobuf) library.
    #[derive(Debug, Clone, Default)]
    pub struct ProtobufCodecV3<T, U> {
        _pd: PhantomData<(T, U)>,
    }

    impl<T, U> Codec for ProtobufCodecV3<T, U>
    where
        T: Message + Send + 'static,
        U: Message + Default + Send + 'static,
    {
        type Encode = T;
        type Decode = U;

        type Encoder = ProtobufEncoderV3<T>;
        type Decoder = ProtobufDecoderV3<U>;

        fn encoder(&mut self) -> Self::Encoder {
            ProtobufEncoderV3 { _pd: PhantomData }
        }

        fn decoder(&mut self) -> Self::Decoder {
            ProtobufDecoderV3 { _pd: PhantomData }
        }
    }

    /// A [`Encoder`] that knows how to encode `T`.
    #[derive(Debug, Clone, Default)]
    pub struct ProtobufEncoderV3<T> {
        _pd: PhantomData<T>,
    }

    impl<T: Message> Encoder for ProtobufEncoderV3<T> {
        type Item = T;
        type Error = Status;

        fn encode(&mut self, item: Self::Item, buf: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
            let mut writer = buf.writer();
            item.write_to_writer(&mut writer)
                .expect("Message only errors if not enough space");

            Ok(())
        }
    }

    /// A [`Decoder`] that knows how to decode `U`.
    #[derive(Debug, Clone, Default)]
    pub struct ProtobufDecoderV3<U> {
        _pd: PhantomData<U>,
    }

    impl<U> ProtobufDecoderV3<U> {
        /// Get a new decoder with explicit buffer settings
        pub fn new() -> Self {
            Self { _pd: PhantomData }
        }
    }

    impl<U: Message + Default> Decoder for ProtobufDecoderV3<U> {
        type Item = U;
        type Error = Status;

        fn decode(&mut self, buf: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
            let mut reader = buf.reader();
            let item = <U as Message>::parse_from_reader(&mut reader).map_err(from_decode_error)?;

            Ok(Some(item))
        }
    }

    fn from_decode_error(error: protobuf::Error) -> Status {
        // Map Protobuf parse errors to an INTERNAL status code, as per
        // https://github.com/grpc/grpc/blob/master/doc/statuscodes.md
        Status::new(Code::Internal, error.to_string())
    }
}

#[cfg(feature = "protobuf-v3")]
pub use protobuf_v3::*;

#[cfg(feature = "protobuf-v2")]
mod protobuf_v2 {
    use std::marker::PhantomData;

    use bytes::{Buf, BufMut};
    use protobuf2::Message;
    use tonic::{
        codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder},
        Code, Status,
    };

    /// A [`Codec`] that implements `application/grpc+proto` via the [rust-protobuf v2](https://crates.io/crates/protobuf/2.28.0) library.
    #[derive(Debug, Clone, Default)]
    pub struct ProtobufCodecV2<T, U> {
        _pd: PhantomData<(T, U)>,
    }

    impl<T, U> Codec for ProtobufCodecV2<T, U>
    where
        T: Message + Send + 'static,
        U: Message + Default + Send + 'static,
    {
        type Encode = T;
        type Decode = U;

        type Encoder = ProtobufEncoderV2<T>;
        type Decoder = ProtobufDecoderV2<U>;

        fn encoder(&mut self) -> Self::Encoder {
            ProtobufEncoderV2 { _pd: PhantomData }
        }

        fn decoder(&mut self) -> Self::Decoder {
            ProtobufDecoderV2 { _pd: PhantomData }
        }
    }

    /// A [`Encoder`] that knows how to encode `T`.
    #[derive(Debug, Clone, Default)]
    pub struct ProtobufEncoderV2<T> {
        _pd: PhantomData<T>,
    }

    impl<T: Message> Encoder for ProtobufEncoderV2<T> {
        type Item = T;
        type Error = Status;

        fn encode(&mut self, item: Self::Item, buf: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
            let mut writer = buf.writer();
            item.write_to_writer(&mut writer)
                .expect("Message only errors if not enough space");

            Ok(())
        }
    }

    /// A [`Decoder`] that knows how to decode `U`.
    #[derive(Debug, Clone, Default)]
    pub struct ProtobufDecoderV2<U> {
        _pd: PhantomData<U>,
    }

    impl<U> ProtobufDecoderV2<U> {
        /// Get a new decoder with explicit buffer settings
        pub fn new() -> Self {
            Self { _pd: PhantomData }
        }
    }

    impl<U: Message + Default> Decoder for ProtobufDecoderV2<U> {
        type Item = U;
        type Error = Status;

        fn decode(&mut self, buf: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
            let mut reader = buf.reader();
            #[allow(deprecated)]
            let item = protobuf2::parse_from_reader(&mut reader).map_err(from_decode_error)?;

            Ok(Some(item))
        }
    }

    fn from_decode_error(error: protobuf2::error::ProtobufError) -> Status {
        // Map Protobuf parse errors to an INTERNAL status code, as per
        // https://github.com/grpc/grpc/blob/master/doc/statuscodes.md
        Status::new(Code::Internal, error.to_string())
    }
}
#[cfg(feature = "protobuf-v2")]
pub use protobuf_v2::*;
