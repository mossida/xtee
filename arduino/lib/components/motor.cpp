#include "motor.hpp"

using namespace components::motor;

void Engine::begin()
{
    engine.init();

    for (size_t i = 0; i < pins::MOTORS_COUNT; i++)
    {
        auto *stepper = engine.stepperConnectToPin(pins::MOTORS[i]->step);

        stepper->setDirectionPin(pins::MOTORS[i]->dir);
        stepper->setEnablePin(pins::MOTORS[i]->enable, false);
        stepper->setAutoEnable(false);

        steppers[i] = stepper;
    }

    protocol->registerHandler(packet::MOVE, this, &Engine::handleMove);
    protocol->registerHandler(packet::KEEP, this, &Engine::handleKeep);
    protocol->registerHandler(packet::SET_SPEED, this, &Engine::handleSetSpeed);
    protocol->registerHandler(packet::SET_ACCELERATION, this, &Engine::handleSetAcceleration);
    protocol->registerHandler(packet::SET_OUTPUTS, this, &Engine::handleSetOutputs);
    protocol->registerHandler(packet::REPORT_STATUS, this, &Engine::handleReportStatus);
    protocol->registerHandler(packet::STOP, this, &Engine::handleStop);

    digitalWriteFast(LED_BUILTIN, LOW);
}

void Engine::handleKeep(const uint8_t *buffer, size_t size)
{
    if (size != sizeof(packet::KEEP_DATA) || buffer[0] < 1 || buffer[0] > pins::MOTORS_COUNT)
        return;

    const packet::KEEP_DATA data = *reinterpret_cast<const packet::KEEP_DATA *>(buffer);

    auto index = data.slave - 1;
    auto *stepper = steppers[index];

    if (data.direction)
        stepper->runForward();
    else
        stepper->runBackward();

    sendStatus(data.slave);
}

void Engine::handleMove(const uint8_t *buffer, size_t size)
{
    if (size != sizeof(packet::MOVE_DATA) || buffer[0] < 1 || buffer[0] > pins::MOTORS_COUNT)
        return;

    const packet::MOVE_DATA data = *reinterpret_cast<const packet::MOVE_DATA *>(buffer);

    auto *stepper = steppers[data.slave - 1];
    auto direction = data.direction ? 1 : -1;
    auto rotations = constrain(data.rotations, 0, settings::MOTOR_ROTATIONS_LIMIT);

    if (rotations == 0)
        return;

    int32_t pulses = settings::MOTOR_STEPS / 10;
    int32_t steps = rotations * direction * pulses;

    stepper->move(steps);

    sendStatus(data.slave);
}

void Engine::handleReportStatus(const uint8_t *buffer, size_t size)
{
    if (size != sizeof(packet::REPORT_STATUS_DATA) || buffer[0] < 1 || buffer[0] > pins::MOTORS_COUNT)
        return;

    const packet::REPORT_STATUS_DATA data = *reinterpret_cast<const packet::REPORT_STATUS_DATA *>(buffer);

    if (!is_initialized)
    {
        // Send recognition for all motors in first call
        for (size_t i = 0; i < pins::MOTORS_COUNT; i++)
            sendRecognition(i + 1);

        is_initialized = true;
    }

    sendStatus(data.slave);
}

void Engine::handleStop(const uint8_t *buffer, size_t size)
{
    if (size != sizeof(packet::STOP_DATA) || buffer[0] < 1 || buffer[0] > pins::MOTORS_COUNT)
        return;

    const packet::STOP_DATA data = *reinterpret_cast<const packet::STOP_DATA *>(buffer);

    auto *stepper = steppers[data.slave - 1];

    if (data.gentle)
        return stepper->stopMove();

    stepper->forceStopAndNewPosition(0);

    sendStatus(data.slave);
}

void Engine::handleSetSpeed(const uint8_t *buffer, size_t size)
{
    if (size != sizeof(packet::SET_SPEED_DATA) || buffer[0] < 1 || buffer[0] > pins::MOTORS_COUNT)
        return;

    const packet::SET_SPEED_DATA data = *reinterpret_cast<const packet::SET_SPEED_DATA *>(buffer);

    auto *stepper = steppers[data.slave - 1];

    stepper->setSpeedInHz(data.speed);

    if (data.apply)
        stepper->applySpeedAcceleration();
}

void Engine::handleSetAcceleration(const uint8_t *buffer, size_t size)
{
    if (size != sizeof(packet::SET_ACCELERATION_DATA) || buffer[0] < 1 || buffer[0] > pins::MOTORS_COUNT)
        return;

    const packet::SET_ACCELERATION_DATA data = *reinterpret_cast<const packet::SET_ACCELERATION_DATA *>(buffer);

    auto *stepper = steppers[data.slave - 1];

    stepper->setAcceleration(data.acceleration);
    stepper->applySpeedAcceleration();
}

void Engine::handleSetOutputs(const uint8_t *buffer, size_t size)
{
    if (size != sizeof(packet::SET_OUTPUTS_DATA) || buffer[0] < 1 || buffer[0] > pins::MOTORS_COUNT)
        return;

    const packet::SET_OUTPUTS_DATA data = *reinterpret_cast<const packet::SET_OUTPUTS_DATA *>(buffer);

    auto *stepper = steppers[data.slave - 1];

    if (data.outputs)
        stepper->enableOutputs();
    else
        stepper->disableOutputs();
}

void Engine::sendStatus(uint8_t slave)
{
    if (slave < 1 || slave > pins::MOTORS_COUNT)
        return;

    auto index = slave - 1;
    auto *stepper = steppers[index];

    const bool outputs = digitalReadFast(pins::MOTORS[index]->enable);

    const packet::STATUS_DATA data = {
        .slave = slave,
        .running = stepper->isRunning(),
        .stopping = stepper->isStopping(),
        .outputs = outputs,
        .position = stepper->getCurrentPosition(),
        .remaining = stepper->stepsToStop()};

    protocol->sendPacket(packet::STATUS, reinterpret_cast<const uint8_t *>(&data), sizeof(data));
}

void Engine::sendRecognition(uint8_t slave)
{
    if (slave < 1 || slave > pins::MOTORS_COUNT)
        return;

    auto index = slave - 1;
    auto *stepper = steppers[index];

    const packet::RECOGNITION_DATA data = {
        .slave = slave,
        .max_speed = stepper->getMaxSpeedInHz()};

    protocol->sendPacket(packet::RECOGNITION, reinterpret_cast<const uint8_t *>(&data), sizeof(data));
}