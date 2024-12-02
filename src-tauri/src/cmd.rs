use ractor::{registry, rpc};
use serialport::SerialPortInfo;

use crate::{
    actor::{
        actuator::ActuatorMessage,
        controller::{Controller, ControllerMessage},
        master::MasterMessage,
    },
    router::RouterContext,
};

#[tauri::command]
pub fn restart() -> Result<(), String> {
    let controller = registry::where_is("controller".to_string())
        .ok_or("Controller not found, how is app living?")?;

    controller.send_message(ControllerMessage::Start).unwrap();

    Ok(())
}

pub fn get_ports(_ctx: RouterContext, _: ()) -> Result<Vec<SerialPortInfo>, rspc::Error> {
    let ports = tokio_serial::available_ports()
        .map_err(|e| rspc::Error::new(rspc::ErrorCode::InternalServerError, e.to_string()))?;
    let ports = ports
        .into_iter()
        .filter(|port| matches!(port.port_type, tokio_serial::SerialPortType::UsbPort(_)))
        .collect();

    Ok(ports)
}

pub async fn get_controllers(_ctx: RouterContext, _: ()) -> Result<Vec<Controller>, rspc::Error> {
    let controller = registry::where_is("master".to_string()).ok_or(rspc::Error::new(
        rspc::ErrorCode::NotFound,
        "Master not found".to_owned(),
    ))?;

    let result = rpc::call(
        &controller,
        |port| MasterMessage::FetchControllers(port),
        None,
    )
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

pub fn actuator_stop(_ctx: RouterContext, _: ()) -> Result<(), rspc::Error> {
    let actuator = registry::where_is("actuator".to_string()).ok_or(rspc::Error::new(
        rspc::ErrorCode::NotFound,
        "Actuator not found".to_owned(),
    ))?;

    actuator
        .send_message(ActuatorMessage::Stop)
        .map_err(|e| rspc::Error::new(rspc::ErrorCode::ClientClosedRequest, e.to_string()))
}
