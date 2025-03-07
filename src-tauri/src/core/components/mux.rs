use std::pin::Pin;

use futures::{stream::SplitSink, SinkExt, Stream};
use ractor::{Actor, ActorProcessingErr, ActorRef};
use ractor_actors::streams::{mux_stream, StreamMuxConfiguration, StreamMuxNotification, Target};

use crate::core::protocol::{Codec, Packet, Protocol};
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio_util::codec::Framed;
use tracing::{debug, error};

pub type MuxSink = SplitSink<Framed<SerialStream, Codec>, Packet>;
pub type MuxStream = Pin<Box<dyn Stream<Item = Packet> + Send + 'static>>;
pub type MuxTarget = Box<dyn Target<MuxStream>>;

#[derive(Debug)]
pub struct Mux;

#[derive(Debug)]
pub enum MuxMessage {
    Write(Packet),
}

pub struct MuxState {
    writer: MuxSink,
}

pub struct MuxArguments {
    pub port: String,
    pub baud_rate: u32,
    pub targets: Vec<MuxTarget>,
}

pub struct MuxCallback;

impl StreamMuxNotification for MuxCallback {
    #[inline]
    fn target_failed(&self, target: String, _err: ActorProcessingErr) {
        error!("Target failed: {}", target);
    }

    #[inline]
    fn end_of_stream(&self) {
        debug!("End of stream - waiting for more data");
    }
}

impl Actor for Mux {
    type Msg = MuxMessage;
    type State = MuxState;
    type Arguments = MuxArguments;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        MuxArguments {
            port,
            baud_rate,
            targets,
        }: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let io_stream = tokio_serial::new(port, baud_rate).open_native_async()?;

        let protocol = Protocol::new(io_stream);
        let (sink, stream) = protocol.framed().await?;

        mux_stream(
            StreamMuxConfiguration {
                stream,
                targets,
                callback: MuxCallback,
                stop_processing_target_on_failure: false,
            },
            Some(myself.get_cell()),
        )
        .await?;

        Ok(MuxState { writer: sink })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        MuxMessage::Write(packet): Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        state
            .writer
            .send(packet)
            .await
            .map_err(ActorProcessingErr::from)
    }

    async fn post_stop(
        &self,
        myself: ActorRef<Self::Msg>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        myself.stop_children(None);
        state.writer.close().await?;

        Ok(())
    }
}
