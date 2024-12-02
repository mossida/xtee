use ractor::registry;
use serialport::SerialPortInfo;

use crate::{
    actor::{
        actuator::ActuatorMessage,
        controller::ControllerMessage,
        motor::{MotorMessage, MotorMovement},
    },
    router::RouterContext,
};

#[tauri::command]
pub fn motor_spin(slave: u8, direction: u8, rotations: u16, speed: u16) -> Result<(), String> {
    let motor = registry::where_is(format!("motor-{}", slave)).ok_or("Motor not found")?;

    motor
        .send_message(MotorMessage::Spin(MotorMovement {
            direction,
            rotations,
            speed,
        }))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restart() -> Result<(), String> {
    let controller = registry::where_is("controller".to_string())
        .ok_or("Controller not found, how is app living?")?;

    controller.send_message(ControllerMessage::Start).unwrap();

    Ok(())
}

#[tauri::command()]
pub fn get_controllers() -> Result<Vec<SerialPortInfo>, String> {
    let ports = tokio_serial::available_ports().map_err(|e| e.to_string())?;
    let ports = ports
        .into_iter()
        .filter(|port| matches!(port.port_type, tokio_serial::SerialPortType::UsbPort(_)))
        .collect();

    Ok(ports)
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
