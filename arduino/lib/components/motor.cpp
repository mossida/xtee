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

void Engine::handleKeep(const uint8_t *data, size_t size)
{
    if (size < 2 || data[0] < 1 || data[0] > pins::MOTORS_COUNT)
        return;

    if (data[1] != 0x01 && data[1] != 0x00)
        return;

    auto index = data[0] - 1;
    auto direction = data[1];
    auto *stepper = steppers[index];

    if (direction == 0x01)
        stepper->runForward();
    else
        stepper->runBackward();

    sendStatus(data[0]);
}

void Engine::handleMove(const uint8_t *data, size_t size)
{
    if (size < 4 || data[0] < 1 || data[0] > pins::MOTORS_COUNT)
        return;

    if (data[1] != 0x00 && data[1] != 0x01)
        return;

    auto index = data[0] - 1;
    auto direction = (2 * data[1]) - 1;
    auto *stepper = steppers[index];
    auto rotations = (uint16_t)data[3] << 8 | data[2];

    if (rotations == 0)
        return;

    // NOTE: multiplication overflow if rotations is uint16_t
    auto steps = settings::MOTOR_STEPS * rotations * direction;

    stepper->move(static_cast<int32_t>(steps));

    sendStatus(data[0]);
}

void Engine::handleReportStatus(const uint8_t *data, size_t size)
{
    if (size < 1 || data[0] < 1 || data[0] > pins::MOTORS_COUNT)
        return;

    if (!is_initialized)
    {
        // Send recognition for all motors in first call
        for (size_t i = 0; i < pins::MOTORS_COUNT; i++)
            sendRecognition(i + 1);

        is_initialized = true;
    }

    sendStatus(data[0]);
}

void Engine::handleStop(const uint8_t *data, size_t size)
{
    if (size < 1 || data[0] < 1 || data[0] > pins::MOTORS_COUNT)
        return;

    if (data[1] != 0x00 && data[1] != 0x01)
        return;

    auto index = data[0] - 1;
    auto mode = data[1];
    auto *stepper = steppers[index];

    if (mode == 0x01)
        return stepper->stopMove();

    stepper->forceStopAndNewPosition(0);

    sendStatus(data[0]);
}

void Engine::handleSetSpeed(const uint8_t *data, size_t size)
{
    if (size < 6 || data[0] < 1 || data[0] > pins::MOTORS_COUNT)
        return;

    if (data[5] != 0x00 && data[5] != 0x01)
        return;

    auto index = data[0] - 1;
    auto *stepper = steppers[index];

    auto speed = (uint32_t)data[4] << 24 | (uint32_t)data[3] << 16 | (uint32_t)data[2] << 8 | data[1];
    auto apply = data[5];

    stepper->setSpeedInHz(speed);

    if (apply == 0x01)
        stepper->applySpeedAcceleration();
}

void Engine::handleSetAcceleration(const uint8_t *data, size_t size)
{
    if (size < 5 || data[0] < 1 || data[0] > pins::MOTORS_COUNT)
        return;

    auto index = data[0] - 1;
    auto *stepper = steppers[index];

    auto acceleration = (uint32_t)data[4] << 24 | (uint32_t)data[3] << 16 | (uint32_t)data[2] << 8 | data[1];

    stepper->setAcceleration(acceleration);

    // TODO: understand if this can be set with a flag
    stepper->applySpeedAcceleration();
}

void Engine::handleSetOutputs(const uint8_t *data, size_t size)
{
    if (size < 2 || data[0] < 1 || data[0] > pins::MOTORS_COUNT)
        return;

    if (data[1] != 0x00 && data[1] != 0x01)
        return;

    auto index = data[0] - 1;
    auto outputs = data[1];
    auto *stepper = steppers[index];

    if (outputs == 0x01)
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

    uint8_t buffer[11];
    int32_t position = stepper->getCurrentPosition();
    uint32_t remaining = stepper->stepsToStop();

    buffer[0] = slave;
    buffer[1] = stepper->isRunning() ? 1 : 0;
    buffer[2] = stepper->isStopping() ? 1 : 0;

    // Write position bytes (32-bit integer) in little-endian
    buffer[3] = position & 0xFF; // Least significant byte first
    buffer[4] = (position >> 8) & 0xFF;
    buffer[5] = (position >> 16) & 0xFF;
    buffer[6] = (position >> 24) & 0xFF; // Most significant byte last

    // Write remaining steps (32-bit integer) in little-endian
    buffer[7] = remaining & 0xFF;
    buffer[8] = (remaining >> 8) & 0xFF;
    buffer[9] = (remaining >> 16) & 0xFF;
    buffer[10] = (remaining >> 24) & 0xFF;

    protocol->sendPacket(packet::STATUS, buffer, 11);
}

void Engine::sendRecognition(uint8_t slave)
{
    if (slave < 1 || slave > pins::MOTORS_COUNT)
        return;

    auto index = slave - 1;
    auto *stepper = steppers[index];
    auto max_speed = stepper->getMaxSpeedInHz();

    uint8_t buffer[5];

    buffer[0] = slave;
    buffer[1] = max_speed & 0xFF;
    buffer[2] = (max_speed >> 8) & 0xFF;
    buffer[3] = (max_speed >> 16) & 0xFF;
    buffer[4] = (max_speed >> 24) & 0xFF;

    protocol->sendPacket(packet::RECOGNITION, buffer, 5);
}