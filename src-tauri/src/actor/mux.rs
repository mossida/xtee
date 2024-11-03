use futures::{
    stream::{self, SplitSink},
    SinkExt, StreamExt,
};
use ractor::{async_trait, Actor, ActorCell, ActorProcessingErr, ActorRef};
use ractor_actors::streams::{mux_stream, StreamMuxConfiguration, StreamMuxNotification, Target};

use tokio_serial::SerialStream;
use tokio_util::codec::{Decoder, Framed};

use crate::{
    error::ControllerError,
    protocol::{Packet, Protocol},
};

type Targets<S> = Vec<Box<dyn Target<S>>>;

type Writer = SplitSink<Framed<SerialStream, Protocol>, Packet>;

pub struct Mux;

pub enum MuxMessage {
    Consume(SerialStream),
    Write(Packet),
}

pub struct MuxState {
    writer: Option<Writer>,
    plexer: Option<ActorCell>,
    targets: Targets<SerialStream>,
}

pub struct MuxCallback;

impl StreamMuxNotification for MuxCallback {
    fn target_failed(&self, target: String, err: ActorProcessingErr) {
        todo!()
    }

    fn end_of_stream(&self) {
        todo!()
    }
}

#[async_trait]
impl Actor for Mux {
    type Msg = MuxMessage;
    type State = MuxState;
    type Arguments = Targets<SerialStream>;

    async fn pre_start(
        &self,
        _: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(MuxState {
            plexer: None,
            writer: None,
            targets: args,
        })
    }

    async fn handle(
        &self,
        actor: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            MuxMessage::Consume(serial) => {
                // Stop current multiplexer
                actor.stop_children(None);

                let protocol = Protocol::new();
                let (sink, _framed_stream) = protocol.framed(serial).split();
                //let stream = framed_stream.filter_map(async |r| r.ok());

                state.writer = Some(sink);

                mux_stream(
                    StreamMuxConfiguration {
                        //stream,
                        stream: stream::iter(vec![17, 19]),
                        targets: vec![],
                        callback: MuxCallback,
                        stop_processing_target_on_failure: false,
                    },
                    Some(actor.get_cell()),
                )
                .await?;
            }
            MuxMessage::Write(packet) => {
                let sink = state
                    .writer
                    .as_mut()
                    .ok_or(ControllerError::OptionMismatch)?;

                sink.send(packet).await?;
            }
        }

        Ok(())
    }
}
