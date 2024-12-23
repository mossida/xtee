use crc::{Algorithm, Crc};
use deku::prelude::*;
use futures::{future, SinkExt, StreamExt};
use serde::Serialize;
use tokio_serial::SerialStream;
use tokio_util::{
    bytes::{BufMut, BytesMut},
    codec::{Decoder, Encoder},
};
use tracing::{error, info, trace};

use crate::{
    components::mux::{MuxSink, MuxStream},
    error::Error,
};

#[derive(Clone, Debug, PartialEq, DekuRead, DekuWrite, Serialize)]
#[deku(id_type = "u8", endian = "little")]
pub enum Packet {
    #[deku(id = 0x01)]
    Ready,
    #[deku(id = 0x02)]
    Data { value: i32 },
    #[deku(id = 0x03)]
    MotorMove {
        slave: u8,
        direction: bool,
        rotations: u16,
    },
    #[deku(id = 0x05)]
    MotorSetSpeed { slave: u8, apply: u8, speed: u32 },
    #[deku(id = 0x06)]
    MotorSetAcceleration { slave: u8, acceleration: u32 },
    #[deku(id = 0x07)]
    MotorSetOutputs { slave: u8, outputs: bool },
    #[deku(id = 0x09)]
    MotorAskStatus { slave: u8 },
    #[deku(id = 0x0A)]
    MotorStatus {
        slave: u8,
        running: bool,
        stopping: bool,
        position: i32,
        remaining: u32,
    },
    #[deku(id = 0x08)]
    MotorRecognition { slave: u8, max_speed: u32 },
    #[deku(id = 0x04)]
    MotorKeep { slave: u8, direction: bool },
    #[deku(id = 0x0B)]
    MotorStop { slave: u8, gentle: bool },
    #[deku(id = 0x0C)]
    ActuatorMove { direction: bool },
    #[deku(id = 0x0D)]
    ActuatorStop,
}

// Optimized Protocol implementation
pub struct Protocol {
    stream: SerialStream,
}

impl Protocol {
    #[inline]
    async fn transform(self) -> Result<(MuxSink, MuxStream), Error> {
        let codec = Codec::new();
        let (mut framed_sink, framed_stream) = codec.framed(self.stream).split();

        // Wait for the Ready packet
        let mut stream = framed_stream.filter_map(|r| future::ready(r.ok()));
        while let Some(packet) = stream.next().await {
            if matches!(packet, Packet::Ready) {
                info!("Received Ready packet, sending acknowledgement");
                framed_sink.send(Packet::Ready).await?;
                break;
            }
        }

        Ok((framed_sink, stream.boxed()))
    }

    #[inline]
    pub fn new(stream: SerialStream) -> Self {
        Self { stream }
    }

    #[inline]
    pub async fn framed(self) -> Result<(MuxSink, MuxStream), Error> {
        self.transform().await
    }
}

pub struct Codec {
    cobs_codec: cobs_codec::Codec<0x00, 0x00, 256, 256>,
    crc: Crc<u8>,
    buffer: BytesMut,
}

impl Codec {
    #[inline]
    pub fn new() -> Self {
        const CRC_ALGORITHM: Algorithm<u8> = Algorithm {
            width: 8,
            poly: 0x9B,
            init: 0x00,
            refin: false,
            refout: false,
            xorout: 0x00,
            check: 0xEA,
            residue: 0x00,
        };

        Self {
            crc: Crc::<u8>::new(&CRC_ALGORITHM),
            cobs_codec: cobs_codec::Codec::new(),
            buffer: BytesMut::with_capacity(256), // Pre-allocate with reasonable size
        }
    }

    #[inline]
    fn validate_crc(&self, data: &[u8], received_crc: u8) -> bool {
        self.crc.checksum(data) == received_crc
    }
}

impl Decoder for Codec {
    type Item = Packet;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.cobs_codec
            .decode(src)
            .map_err(|_| Error::Packet)
            .and_then(|decoded| match decoded {
                Some(ref decoded_data) if decoded_data.len() >= 2 => {
                    let size = decoded_data.len() - 1;
                    let (data, crc_byte) = decoded_data.split_at(size);

                    if !self.validate_crc(data, crc_byte[0]) {
                        error!(
                            "CRC mismatch, calculated: {}, received: {}",
                            self.crc.checksum(data),
                            crc_byte[0]
                        );
                        return Ok(None);
                    }

                    Packet::from_bytes((data, 0))
                        .map(|(_, packet)| Some(packet))
                        .map_err(|_| Error::Packet)
                }
                _ => Ok(None),
            })
    }
}

impl Encoder<Packet> for Codec {
    type Error = Error;

    fn encode(&mut self, packet: Packet, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.buffer.clear();

        // Serialize directly into our reusable buffer
        packet
            .to_bytes()
            .map_err(|_| Error::Packet)
            .and_then(|packet_bytes| {
                let crc = self.crc.checksum(&packet_bytes);

                self.buffer.extend_from_slice(&packet_bytes);
                self.buffer.put_u8(crc);

                self.cobs_codec
                    .encode(&self.buffer, dst)
                    .map_err(|_| Error::Packet)
            })
            .inspect_err(|e| error!("Encoding error: {:?}", e))
            .map(|_| {
                trace!("Encoded packet size: {}", dst.len());
            })
    }
}
