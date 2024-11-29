use ractor::registry;
use serialport::SerialPortInfo;

use crate::actor::{
    actuator::ActuatorMessage,
    controller::ControllerMessage,
    motor::{MotorMessage, MotorMovement},
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

#[tauri::command]
pub fn actuator_load(setpoint: f32) -> Result<(), String> {
    let actor = registry::where_is("actuator".to_string()).ok_or("Actuator not found")?;

    actor
        .send_message(ActuatorMessage::Load(setpoint))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn actuator_keep(setpoint: f32) -> Result<(), String> {
    let actor = registry::where_is("actuator".to_string()).ok_or("Actuator not found")?;

    actor
        .send_message(ActuatorMessage::Keep(setpoint))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn actuator_move(direction: u8) -> Result<(), String> {
    let actuator = registry::where_is("actuator".to_string()).ok_or("Actuator not found")?;

    actuator
        .send_message(ActuatorMessage::Move(direction))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn actuator_stop() -> Result<(), String> {
    let actuator = registry::where_is("actuator".to_string()).ok_or("Actuator not found")?;

    actuator
        .send_message(ActuatorMessage::Stop)
        .map_err(|e| e.to_string())
}
