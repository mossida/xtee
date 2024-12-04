use std::{any::TypeId, pin::Pin};

use futures::{stream::SplitSink, SinkExt, Stream};
use ractor::{async_trait, pg, Actor, ActorCell, ActorProcessingErr, ActorRef};
use ractor_actors::streams::{mux_stream, StreamMuxConfiguration, StreamMuxNotification, Target};

use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio_util::codec::Framed;
use tracing::{error, info};

use crate::components::master::SCOPE;
use crate::{
    error::ControllerError,
    protocol::{Codec, Packet, Protocol},
};

use super::controller::Controller;
use super::{actuator::ActuatorMessage, motor::MotorMessage};

pub type MuxSink = SplitSink<Framed<SerialStream, Codec>, Packet>;
pub type MuxStream = Pin<Box<dyn Stream<Item = Packet> + Send + 'static>>;

pub struct Mux;

pub enum MuxMessage {
    Write(Packet),
}

pub struct MuxState {
    writer: MuxSink,
}

pub struct MuxArguments {
    stream: SerialStream,
    controller: Controller,
}

impl TryFrom<Controller> for MuxArguments {
    type Error = ControllerError;

    fn try_from(controller: Controller) -> Result<Self, Self::Error> {
        let stream = tokio_serial::new(controller.serial_port.clone(), controller.baud_rate)
            .open_native_async()?;

        Ok(MuxArguments { stream, controller })
    }
}

pub struct MuxTarget {
    cell: ActorCell,
}

impl From<ActorCell> for MuxTarget {
    fn from(cell: ActorCell) -> Self {
        MuxTarget { cell }
    }
}

impl Target<MuxStream> for MuxTarget {
    fn get_id(&self) -> String {
        self.cell.get_id().to_string()
    }

    fn message_received(&self, item: Packet) -> Result<(), ActorProcessingErr> {
        match self.cell.get_type_id() {
            t if t == TypeId::of::<ActuatorMessage>() => {
                self.cell.send_message(ActuatorMessage::from(item))?;
            }

            t if t == TypeId::of::<MotorMessage>() => {
                self.cell.send_message(MotorMessage::from(item))?;
            }
            _ => {}
        }

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
        let protocol = Protocol::new(args.stream);
        let (sink, stream) = protocol.framed(myself.clone());

        let targets: Vec<_> =
            pg::get_scoped_members(&SCOPE.to_owned(), &args.controller.group.into())
                .into_iter()
                .map(|child| {
                    Box::new(MuxTarget::from(child))
                        as Box<dyn ractor_actors::streams::Target<MuxStream>>
                })
                .collect();

        mux_stream(
            StreamMuxConfiguration {
                stream,
                targets,
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
