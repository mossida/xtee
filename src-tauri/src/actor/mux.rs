use futures::{future, stream::SplitSink, SinkExt, StreamExt};
use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef};
use ractor_actors::streams::{mux_stream, StreamMuxConfiguration, StreamMuxNotification, Target};

use tokio_serial::SerialStream;
use tokio_util::codec::{Decoder, Framed};
use tracing::{error, info};

use crate::protocol::{Packet, Protocol};

type Targets<S> = Vec<Box<dyn Target<S>>>;

type Writer = SplitSink<Framed<SerialStream, Protocol>, Packet>;

pub struct Mux;

pub enum MuxMessage {
    Write(Packet),
}

pub struct MuxState {
    writer: Writer,
}

pub struct MuxArguments {
    stream: SerialStream,
    targets: Targets<Packet>,
}

pub struct MuxCallback;

impl StreamMuxNotification for MuxCallback {
    fn target_failed(&self, target: String, _err: ActorProcessingErr) {
        error!("Target failed: {}", target);
    }

    fn end_of_stream(&self) {
        info!("End of stream");
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
        let protocol = Protocol::new();
        let (sink, framed_stream) = protocol.framed(args.stream).split();
        let stream = framed_stream.filter_map(|r| future::ready(r.ok()));

        mux_stream(
            StreamMuxConfiguration {
                stream,
                // TODO: Add targets
                targets: vec![],
                callback: MuxCallback,
                stop_processing_target_on_failure: true,
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
        match message {
            MuxMessage::Write(packet) => {
                state.writer.send(packet).await?;
            }
        }

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
