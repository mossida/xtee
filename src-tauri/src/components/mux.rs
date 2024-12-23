use std::{any::TypeId, pin::Pin};

use futures::{stream::SplitSink, SinkExt, Stream};
use ractor::{async_trait, Actor, ActorCell, ActorProcessingErr, ActorRef};
use ractor_actors::streams::{mux_stream, StreamMuxConfiguration, StreamMuxNotification, Target};

use crate::{
    error::Error,
    protocol::{Codec, Packet, Protocol},
};
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio_util::codec::Framed;
use tracing::{debug, error};

use super::controller::Controller;
use super::{actuator::ActuatorMessage, motor::MotorMessage};

// Use static dispatch for better performance
pub type MuxSink = SplitSink<Framed<SerialStream, Codec>, Packet>;
pub type MuxStream = Pin<Box<dyn Stream<Item = Packet> + Send + 'static>>;

#[derive(Debug)]
pub struct Mux;

#[derive(Debug)]
pub enum MuxMessage {
    Write(Packet),
}

pub struct MuxState {
    writer: MuxSink,
    reader: Option<MuxStream>,
}

#[derive(Debug)]
pub struct MuxArguments {
    stream: SerialStream,
}

impl TryFrom<Controller> for MuxArguments {
    type Error = Error;

    #[inline]
    fn try_from(controller: Controller) -> Result<Self, Self::Error> {
        let stream = tokio_serial::new(controller.serial_port.clone(), controller.baud_rate)
            .open_native_async()?;

        Ok(MuxArguments { stream })
    }
}

// Optimize MuxTarget to use references where possible
pub struct MuxTarget {
    cell: ActorCell,
    type_id: TypeId, // Cache TypeId for better performance
}

impl From<ActorCell> for MuxTarget {
    #[inline]
    fn from(cell: ActorCell) -> Self {
        MuxTarget {
            type_id: cell.get_type_id(),
            cell,
        }
    }
}

impl Target<MuxStream> for MuxTarget {
    #[inline]
    fn get_id(&self) -> String {
        self.cell.get_id().to_string()
    }

    #[inline]
    fn message_received(&self, item: Packet) -> Result<(), ActorProcessingErr> {
        // Use cached TypeId for faster matching

        if self.type_id == TypeId::of::<ActuatorMessage>() {
            self.cell.send_message(ActuatorMessage::from(item))?;
        } else if self.type_id == TypeId::of::<MotorMessage>() {
            self.cell.send_message(MotorMessage::from(item))?;
        }

        Ok(())
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
        _myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let protocol = Protocol::new(args.stream);
        let (sink, stream) = protocol.framed().await?;

        Ok(MuxState {
            writer: sink,
            reader: Some(stream),
        })
    }

    async fn post_start(
        &self,
        myself: ActorRef<Self::Msg>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        let supervisor = myself.try_get_supervisor().ok_or(Error::Config)?;
        let children = supervisor.get_children();

        // Pre-allocate vector with capacity
        let mut targets = Vec::with_capacity(children.len());

        targets.extend(
            children
                .into_iter()
                .filter(|child| child.get_type_id() != TypeId::of::<MuxMessage>())
                .inspect(|child| debug!("Multiplexing to {}", child.get_name().unwrap_or_default()))
                .map(|child| {
                    Box::new(MuxTarget::from(child))
                        as Box<dyn ractor_actors::streams::Target<MuxStream>>
                }),
        );

        mux_stream(
            StreamMuxConfiguration {
                stream: state.reader.take().expect("Reader already taken"),
                targets,
                callback: MuxCallback::default(),
                stop_processing_target_on_failure: false,
            },
            Some(myself.get_cell()),
        )
        .await?;

        Ok(())
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
