use std::pin::Pin;

use futures::{stream::SplitSink, SinkExt, Stream};
use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef};
use ractor_actors::streams::{mux_stream, StreamMuxConfiguration, StreamMuxNotification, Target};

use crate::{
    error::Error,
    protocol::{Codec, Packet, Protocol},
};
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio_util::codec::Framed;
use tracing::{debug, error};

use super::controller::Controller;

// Use static dispatch for better performance
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
    stream: SerialStream,
    targets: Vec<MuxTarget>,
}

impl TryFrom<(Controller, Vec<MuxTarget>)> for MuxArguments {
    type Error = Error;

    #[inline]
    fn try_from((controller, targets): (Controller, Vec<MuxTarget>)) -> Result<Self, Self::Error> {
        let stream = tokio_serial::new(controller.serial_port.clone(), controller.baud_rate)
            .open_native_async()?;

        Ok(MuxArguments { stream, targets })
    }
}

// Optimize MuxCallback with static dispatch
#[derive(Debug, Default)]
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

#[async_trait]
impl Actor for Mux {
    type Msg = MuxMessage;
    type State = MuxState;
    type Arguments = MuxArguments;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let protocol = Protocol::new(args.stream);
        let (sink, stream) = protocol.framed().await?;

        mux_stream(
            StreamMuxConfiguration {
                stream,
                targets: args.targets,
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
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        let MuxMessage::Write(packet) = message;
        state.writer.send(packet).await?;

        Ok(())
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
