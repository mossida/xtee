use crc::{Algorithm, Crc};
use deku::prelude::*;
use futures::{future, StreamExt};
use ractor::ActorRef;
use serde::Serialize;
use tokio::io::{self};
use tokio_serial::SerialStream;
use tokio_util::{
    bytes::BytesMut,
    codec::{Decoder, Encoder},
};
use tracing::{error, trace};

use crate::{
    actor::mux::{MuxMessage, MuxSink, MuxStream},
    error::ControllerError,
};

// control flow

pub const READY_ID: u8 = 0x01;
pub const DATA_ID: u8 = 0x02;
pub const HEALTH_REQUEST_ID: u8 = 0x03;
pub const HEALTH_RESPONSE_ID: u8 = 0x04;
pub const MOTOR_COMMAND_ID: u8 = 0x05;
pub const MOTOR_STOP_ID: u8 = 0x06;
pub const ACTUATOR_MOVE_ID: u8 = 0x07;
pub const ACTUATOR_STOP_ID: u8 = 0x08;
pub const TARE_CELL_ID: u8 = 0x09;
pub const TARE_SUCCESS_ID: u8 = 0x0a;

#[derive(Clone, Debug, PartialEq, DekuRead, DekuWrite, Serialize)]
#[deku(id_type = "u8", endian = "little")]
pub enum Packet {
    #[deku(id = "READY_ID")]
    Ready,
    #[deku(id = "DATA_ID")]
    Data { value: i32 },
    #[deku(id = "HEALTH_REQUEST_ID")]
    HealthRequest,
    #[deku(id = "HEALTH_RESPONSE_ID")]
    HealthResponse { uptime: u32, status: u8 },
    #[deku(id = "MOTOR_COMMAND_ID")]
    MotorCommand {
        slave: u8,
        direction: u8,
        revolutions: u16,
        speed: u16,
    },
    #[deku(id = "MOTOR_STOP_ID")]
    MotorStop { slave: u8 },
    #[deku(id = "ACTUATOR_MOVE_ID")]
    ActuatorMove { direction: u8 },
    #[deku(id = "ACTUATOR_STOP_ID")]
    ActuatorStop,
    #[deku(id = "TARE_CELL_ID")]
    TareCell,
    #[deku(id = "TARE_SUCCESS_ID")]
    TareSuccess,
}

pub struct Protocol {
    stream: SerialStream,
}

impl Protocol {
    fn transform(self, mux: ActorRef<MuxMessage>) -> (MuxSink, MuxStream) {
        let codec = Codec::new();
        let (framed_sink, framed_stream) = codec.framed(self.stream).split();

        let mux = mux.clone();
        let stream = framed_stream
            .filter_map(|r| future::ready(r.ok()))
            .then(move |packet| {
                trace!("Received packet: {:?}", packet);

                let mux = mux.clone();

                async move {
                    match packet {
                        Packet::Ready => {
                            mux.send_message(MuxMessage::Write(Packet::Ready)).unwrap()
                        }
                        _ => {}
                    }

                    packet
                }
            });

        (framed_sink, stream.boxed())
    }

    pub fn new(stream: SerialStream) -> Self {
        Self { stream }
    }

    pub fn framed(self, mux: ActorRef<MuxMessage>) -> (MuxSink, MuxStream) {
        Self::transform(self, mux)
    }
}

pub struct Codec {
    cobs_codec: cobs_codec::Codec<0x00, 0x00, 256, 256>,
    crc: Crc<u8>,
}

impl Codec {
    pub fn new() -> Self {
        let crc = Crc::<u8>::new(&Algorithm {
            width: 8,      // 8-bit CRC
            poly: 0x9B,    // Polynomial used for CRC calculation
            init: 0x00,    // Initial value for the CRC register
            refin: false,  // Input data is not reflected
            refout: false, // Output CRC is not reflected
            xorout: 0x00,  // No XOR applied to the final CRC
            check: 0xEA,   // Precomputed "check" value for "123456789"
            residue: 0x00, // Residue for correct packets
        });

        Self {
            crc,
            cobs_codec: cobs_codec::Codec::new(),
        }
    }
}

impl Decoder for Codec {
    type Item = Packet;
    type Error = ControllerError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(decoded) = self
            .cobs_codec
            .decode(src)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
        {
            if decoded.len() < 2 {
                return Ok(None);
            }

            let size = decoded.len() - 1;
            let data = &decoded[..size];

            let received_crc = u8::from_le(decoded[size]);
            let calculated_crc = self.crc.checksum(data);

            if calculated_crc != received_crc {
                error!(
                    "CRC mismatch, expected: {}, received: {}. Packet: {:?}",
                    calculated_crc,
                    received_crc,
                    decoded.to_vec()
                );
                return Ok(None);
            }

            match Packet::from_bytes((data, 0)) {
                Ok((_, packet)) => Ok(Some(packet)),
                Err(err) => {
                    error!("Packet error: {:?}", err);
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }
}

impl Encoder<Packet> for Codec {
    type Error = ControllerError;

    fn encode(&mut self, packet: Packet, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Serialize the packet
        let packet_bytes = packet
            .to_bytes()
            .map_err(|_| ControllerError::PacketError)?;

        // Calculate CRC
        let crc = self.crc.checksum(&packet_bytes);

        // Create a buffer for the frame (packet + CRC)
        let mut frame_buffer = Vec::with_capacity(packet_bytes.len() + 2);

        frame_buffer.extend(packet_bytes);
        frame_buffer.extend(crc.to_le_bytes());

        self.cobs_codec
            .encode(&frame_buffer, dst)
            .map_err(|_| ControllerError::PacketError)?;

        Ok(())
    }
}
