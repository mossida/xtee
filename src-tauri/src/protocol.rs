use cobs_codec::Codec;
use crc::{Crc, CRC_16_MODBUS};
use deku::prelude::*;
use serde::Serialize;
use tokio::io;
use tokio_util::{
    bytes::BytesMut,
    codec::{Decoder, Encoder},
};

pub const DATA_ID: u8 = 0x00;
pub const HEALTH_REQUEST_ID: u8 = 0x01;
pub const HEALTH_RESPONSE_ID: u8 = 0x02;
pub const MOTOR_COMMAND_ID: u8 = 0x03;
pub const MOTOR_STOP_ID: u8 = 0x04;
pub const TARE_CELL_ID: u8 = 0x05;
pub const TARE_SUCCESS_ID: u8 = 0x06;

#[derive(Clone, Debug, PartialEq, DekuRead, DekuWrite, Serialize)]
#[deku(id_type = "u8", endian = "big")]
pub enum Packet {
    #[deku(id = "DATA_ID")]
    Data { value: u16 },
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
    #[deku(id = "TARE_CELL_ID")]
    TareCell,
    #[deku(id = "TARE_SUCCESS_ID")]
    TareSuccess,
}

pub struct Protocol {
    codec: Codec,
    crc: Crc<u16>,
}

impl Protocol {
    pub fn new() -> Self {
        Self {
            codec: Codec::new(),
            crc: Crc::<u16>::new(&CRC_16_MODBUS),
        }
    }
}

impl Decoder for Protocol {
    type Item = Packet;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(decoded) = self
            .codec
            .decode(src)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
        {
            if decoded.len() < 3 {
                return Ok(None);
            }

            let data_len = decoded.len() - 2;
            let (data, crc_bytes) = decoded.split_at(data_len);

            let received_crc = u16::from_be_bytes([crc_bytes[0], crc_bytes[1]]);
            let calculated_crc = self.crc.checksum(data);

            if calculated_crc != received_crc {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "CRC mismatch"));
            }

            match Packet::from_bytes((data, 0)) {
                Ok((_, packet)) => Ok(Some(packet)),
                Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string())),
            }
        } else {
            Ok(None)
        }
    }
}

impl Encoder<Packet> for Protocol {
    type Error = io::Error;

    fn encode(&mut self, packet: Packet, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Serialize the packet
        let packet_bytes = packet
            .to_bytes()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Calculate CRC
        let crc = self.crc.checksum(&packet_bytes);

        // Create a buffer for the frame (packet + CRC)
        let mut frame_buffer = Vec::with_capacity(packet_bytes.len() + 2);
        frame_buffer.extend_from_slice(&packet_bytes);
        frame_buffer.extend_from_slice(&crc.to_be_bytes());

        // Reserve space in the destination buffer
        // COBS encoding may add up to 1 byte per 254 bytes, plus 1 overhead byte
        let max_encoded_len = frame_buffer.len() + (frame_buffer.len() / 254) + 1;
        dst.reserve(max_encoded_len);

        // COBS encode the frame (packet + CRC)
        self.codec
            .encode(&frame_buffer, dst)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(())
    }
}
