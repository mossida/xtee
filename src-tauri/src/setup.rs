use cobs_codec::Codec;
use futures::StreamExt;
use time::macros::{format_description, offset};
use tokio::io;
use tokio_serial::SerialPortBuilderExt;
use tokio_util::{
    bytes::BytesMut,
    codec::{Decoder, Encoder},
};
use tracing::Level;
use tracing_subscriber::fmt::time::OffsetTime;

use crate::protocol::Protocol;

pub fn setup_logging() {
    let fmt = if cfg!(debug_assertions) {
        format_description!("[hour]:[minute]:[second].[subsecond digits:3]")
    } else {
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]")
    };

    let timer = OffsetTime::new(offset!(+8), fmt);

    let builder = tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_timer(timer);

    if cfg!(debug_assertions) {
        builder.init();
    } else {
        builder.json().init();
    }
}

struct LineCodec;

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let newline = src.as_ref().iter().position(|b| *b == b'\n');
        if let Some(n) = newline {
            let line = src.split_to(n + 1);
            return match String::from_utf8(line.to_vec()) {
                Ok(s) => Ok(Some(s)),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid String")),
            };
        }
        Ok(None)
    }
}

impl Encoder<String> for LineCodec {
    type Error = io::Error;

    fn encode(&mut self, _item: String, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().to_owned();
    let protocol = Protocol::new();
    tauri::async_runtime::spawn(async move {
        let port = tokio_serial::new("/dev/cu.usbmodem113101", 115200)
            .open_native_async()
            .unwrap();
        let mut reader = protocol.framed(port);

        while let Some(packet) = reader.next().await {
            println!("{:?}", packet);
        }
    });

    //tauri::async_runtime::spawn(async move { Controller::spawn(handle, port, protocol).await });

    Ok(())
}
