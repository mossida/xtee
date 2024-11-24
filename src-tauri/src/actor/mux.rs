use std::{marker::PhantomData, pin::Pin, sync::Arc};

use futures::{
    future::{self},
    stream::SplitSink,
    SinkExt, Stream, StreamExt,
};
use ractor::{async_trait, Actor, ActorCell, ActorProcessingErr, ActorRef};
use ractor_actors::streams::{mux_stream, StreamMuxConfiguration, StreamMuxNotification, Target};

use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio_util::codec::{Decoder, Framed};
use tracing::{error, info};

use crate::{
    error::ControllerError,
    protocol::{Packet, Protocol},
    store::{Store, CONTROLLER_BAUD, CONTROLLER_BUS},
};

type Writer = SplitSink<Framed<SerialStream, Protocol>, Packet>;
pub type MuxStream = Pin<Box<dyn Stream<Item = Packet> + Send + 'static>>;

pub struct Mux;

pub enum MuxMessage {
    Write(Packet),
}

pub struct MuxState {
    writer: Writer,
}

pub struct MuxArguments {
    stream: SerialStream,
    pub targets: Vec<Box<dyn Target<MuxStream>>>,
}

impl TryFrom<Arc<Store>> for MuxArguments {
    type Error = ControllerError;

    fn try_from(value: Arc<Store>) -> Result<Self, Self::Error> {
        let bus_value = value
            .get(CONTROLLER_BUS)
            .ok_or(ControllerError::ConfigError)?;
        let baud_value = value
            .get(CONTROLLER_BAUD)
            .ok_or(ControllerError::ConfigError)?;

        let bus = bus_value.as_str().ok_or(ControllerError::ConfigError)?;
        let baud = baud_value.as_u64().ok_or(ControllerError::ConfigError)?;

        let stream = tokio_serial::new(bus, baud as u32).open_native_async()?;

        Ok(MuxArguments {
            stream,
            targets: vec![], // Will be injected by controller
        })
    }
}

pub struct MuxTarget<S> {
    cell: ActorCell,
    _s: PhantomData<S>,
}

impl<S> From<ActorCell> for MuxTarget<S> {
    fn from(cell: ActorCell) -> Self {
        MuxTarget {
            cell,
            _s: PhantomData,
        }
    }
}

impl<S> Target<S> for MuxTarget<S>
where
    S: Stream + ractor::State,
    S::Item: Clone + ractor::Message,
{
    fn get_id(&self) -> String {
        self.cell.get_id().to_string()
    }

    fn message_received(&self, item: <S as Stream>::Item) -> Result<(), ActorProcessingErr> {
        self.cell.send_message(item)?;

        Ok(())
    }
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
        let stream = framed_stream.filter_map(|r| future::ready(r.ok())).boxed();

        mux_stream(
            StreamMuxConfiguration {
                stream,
                targets: args.targets,
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
