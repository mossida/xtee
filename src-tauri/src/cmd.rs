use futures::Stream;
use ractor::{registry, rpc, ActorRef};
use serde::Deserialize;

use specta::Type;

use crate::{
    components::{
        actuator::ActuatorMessage,
        controller::Controller,
        master::{Event, MasterMessage},
        motor::{MotorMessage, MotorMovement, MotorStatus},
    },
    router::RouterContext,
};

// TODO: fix error handling
pub fn events_bus(_ctx: RouterContext, _: ()) -> impl Stream<Item = Event> {
    async_stream::stream! {
        /*let master: ActorRef<MasterMessage> = registry::where_is("master".to_string()).expect("master to exist").into();

        let stream = master
            .call(MasterMessage::FetchStream, None)
            .await
            .expect("failed to ask master for stream")
            .success_or(rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                "No response from master".to_owned(),
            ))
            .expect("failed to fetch stream from master")
        .filter_map(|e| future::ready(e.ok()));

        for await event in stream {
            yield event;
        }*/
        yield Event::Weight(100.0);
    }
}

pub fn restart(_ctx: RouterContext, _: ()) -> Result<(), rspc::Error> {
    let master = registry::where_is("master".to_string()).ok_or(rspc::Error::new(
        rspc::ErrorCode::NotFound,
        "master not found, how is app living?".to_owned(),
    ))?;

    master
        .send_message(MasterMessage::Restart)
        .map_err(|e| rspc::Error::new(rspc::ErrorCode::ClientClosedRequest, e.to_string()))?;

    Ok(())
}

#[derive(Default, Type, Deserialize)]
#[allow(unused)]
pub struct Port {
    pub name: String,
    pub manufacturer: Option<String>,
    pub serial_number: Option<String>,
}

#[allow(unused)]
pub fn get_ports(_ctx: RouterContext, _: ()) -> Result<Vec<Port>, rspc::Error> {
    let ports = tokio_serial::available_ports()
        .map_err(|e| rspc::Error::new(rspc::ErrorCode::InternalServerError, e.to_string()))?
        .into_iter()
        .filter(|port| matches!(port.port_type, tokio_serial::SerialPortType::UsbPort(_)))
        .map(|port| match port.port_type {
            tokio_serial::SerialPortType::UsbPort(usb) => Port {
                name: port.port_name,
                manufacturer: usb.manufacturer,
                serial_number: usb.serial_number,
            },
            _ => Default::default(),
        })
        .collect();

    Ok(ports)
}

pub fn motor_keep(_ctx: RouterContext, input: (u8, MotorMovement)) -> Result<(), rspc::Error> {
    let (slave, movement) = input;
    let motor = registry::where_is(format!("motor-{}", slave)).ok_or(rspc::Error::new(
        rspc::ErrorCode::NotFound,
        format!("Motor {} not found", slave),
    ))?;

    motor
        .send_message(MotorMessage::Keep(movement))
        .map_err(|e| rspc::Error::new(rspc::ErrorCode::ClientClosedRequest, e.to_string()))?;

    Ok(())
}

pub fn motor_spin(_ctx: RouterContext, input: (u8, MotorMovement)) -> Result<(), rspc::Error> {
    let (slave, movement) = input;
    let motor = registry::where_is(format!("motor-{}", slave)).ok_or(rspc::Error::new(
        rspc::ErrorCode::NotFound,
        format!("Motor {} not found", slave),
    ))?;

    motor
        .send_message(MotorMessage::Spin(movement))
        .map_err(|e| rspc::Error::new(rspc::ErrorCode::ClientClosedRequest, e.to_string()))?;

    Ok(())
}

pub fn motor_set_outputs(_ctx: RouterContext, input: (u8, bool)) -> Result<bool, rspc::Error> {
    let (slave, enabled) = input;
    let motor = registry::where_is(format!("motor-{}", slave)).ok_or(rspc::Error::new(
        rspc::ErrorCode::NotFound,
        format!("Motor {} not found", slave),
    ))?;

    motor
        .send_message(MotorMessage::SetOutputs(enabled))
        .map_err(|e| rspc::Error::new(rspc::ErrorCode::ClientClosedRequest, e.to_string()))?;

    Ok(enabled)
}

pub async fn motor_get_max_speed(_ctx: RouterContext, slave: u8) -> Result<u32, rspc::Error> {
    let motor: ActorRef<MotorMessage> = registry::where_is(format!("motor-{}", slave))
        .ok_or(rspc::Error::new(
            rspc::ErrorCode::NotFound,
            format!("Motor {} not found", slave),
        ))?
        .into();

    motor
        .call(MotorMessage::GetMaxSpeed, None)
        .await
        .map_err(|e| rspc::Error::new(rspc::ErrorCode::InternalServerError, e.to_string()))?
        .success_or(rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            "No response from motor".to_owned(),
        ))
}

#[derive(Type, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MotorStopMode {
    Graceful,
    Emergency,
}

pub fn motor_stop(_ctx: RouterContext, input: (u8, MotorStopMode)) -> Result<(), rspc::Error> {
    let (slave, mode) = input;
    let motor = registry::where_is(format!("motor-{}", slave)).ok_or(rspc::Error::new(
        rspc::ErrorCode::NotFound,
        format!("Motor {} not found", slave),
    ))?;

    motor
        .send_message(if mode == MotorStopMode::Graceful {
            MotorMessage::GracefulStop
        } else {
            MotorMessage::EmergencyStop
        })
        .map_err(|e| rspc::Error::new(rspc::ErrorCode::ClientClosedRequest, e.to_string()))?;

    Ok(())
}

pub async fn get_controllers(_ctx: RouterContext, _: ()) -> Result<Vec<Controller>, rspc::Error> {
    let controller = registry::where_is("master".to_string()).ok_or(rspc::Error::new(
        rspc::ErrorCode::NotFound,
        "Master not found".to_owned(),
    ))?;

    let result = rpc::call(&controller, MasterMessage::FetchControllers, None)
        .await
        .map_err(|e| rspc::Error::new(rspc::ErrorCode::InternalServerError, e.to_string()))?
        .success_or(rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            "No response from master".to_owned(),
        ))?;

    Ok(result)
}

pub fn actuator_load(_ctx: RouterContext, setpoint: f32) -> Result<(), rspc::Error> {
    let actor = registry::where_is("actuator".to_string()).ok_or(rspc::Error::new(
        rspc::ErrorCode::NotFound,
        "Actuator not found".to_owned(),
    ))?;

    actor
        .send_message(ActuatorMessage::Load(setpoint))
        .map_err(|e| rspc::Error::new(rspc::ErrorCode::ClientClosedRequest, e.to_string()))
}

pub fn actuator_keep(_ctx: RouterContext, setpoint: f32) -> Result<(), rspc::Error> {
    let actor = registry::where_is("actuator".to_string()).ok_or(rspc::Error::new(
        rspc::ErrorCode::NotFound,
        "Actuator not found".to_owned(),
    ))?;

    actor
        .send_message(ActuatorMessage::Keep(setpoint))
        .map_err(|e| rspc::Error::new(rspc::ErrorCode::ClientClosedRequest, e.to_string()))
}

pub fn actuator_move(_ctx: RouterContext, direction: u8) -> Result<(), rspc::Error> {
    let actuator = registry::where_is("actuator".to_string()).ok_or(rspc::Error::new(
        rspc::ErrorCode::NotFound,
        "Actuator not found".to_owned(),
    ))?;

    actuator
        .send_message(ActuatorMessage::Move(direction))
        .map_err(|e| rspc::Error::new(rspc::ErrorCode::ClientClosedRequest, e.to_string()))
}

pub fn actuator_tune(_ctx: RouterContext, _: ()) -> Result<(), rspc::Error> {
    let actuator = registry::where_is("actuator".to_string()).ok_or(rspc::Error::new(
        rspc::ErrorCode::NotFound,
        "Actuator not found".to_owned(),
    ))?;

    actuator
        .send_message(ActuatorMessage::Tune)
        .map_err(|e| rspc::Error::new(rspc::ErrorCode::ClientClosedRequest, e.to_string()))
}

pub fn actuator_stop(_ctx: RouterContext, _: ()) -> Result<(), rspc::Error> {
    let actuator = registry::where_is("actuator".to_string()).ok_or(rspc::Error::new(
        rspc::ErrorCode::NotFound,
        "Actuator not found".to_owned(),
    ))?;

    actuator
        .send_message(ActuatorMessage::Stop)
        .map_err(|e| rspc::Error::new(rspc::ErrorCode::ClientClosedRequest, e.to_string()))
}
