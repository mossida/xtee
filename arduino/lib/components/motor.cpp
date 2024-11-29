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

        steppers[i] = stepper;
    }

    protocol->registerHandler(packet::MOVE, this, &Engine::handleMove);
    protocol->registerHandler(packet::SETTINGS, this, &Engine::handleSettings);
    protocol->registerHandler(packet::REPORT_STATUS, this, &Engine::handleReportStatus);
    protocol->registerHandler(packet::STOP, this, &Engine::handleStop);
}

void Engine::handleMove(const uint8_t *data, size_t size)
{
    if (size < 1 || data[0] < 1 || data[0] > pins::MOTORS_COUNT)
        return;

    auto index = data[0] - 1;
    auto *stepper = steppers[index];
    auto direction = data[1];

    auto rotations = (uint16_t)data[2] << 8 | data[3];

    stepper->move(direction == 0x01 ? rotations : -rotations);

    sendStatus(data[0]);
}

void Engine::handleReportStatus(const uint8_t *data, size_t size)
{
    if (size < 1 || data[0] < 1 || data[0] > pins::MOTORS_COUNT)
        return;

    sendStatus(data[0]);
}

void Engine::handleStop(const uint8_t *data, size_t size)
{
    if (size < 1 || data[0] < 1 || data[0] > pins::MOTORS_COUNT)
        return;

    auto index = data[0] - 1;
    auto *stepper = steppers[index];
    auto mode = data[1];

    if (mode == 0x01)
        return stepper->stopMove();

    stepper->forceStop();
}

void Engine::handleSettings(const uint8_t *data, size_t size)
{
    if (size < 5 || data[0] < 1 || data[0] > pins::MOTORS_COUNT)
        return;

    auto index = data[0] - 1;
    auto *stepper = steppers[index];

    auto speed = (uint16_t)data[1] << 8 | data[2];
    auto acceleration = (uint16_t)data[3] << 8 | data[4];

    stepper->setSpeedInHz(speed);
    stepper->setAcceleration(acceleration);

    stepper->applySpeedAcceleration();
}

void Engine::sendStatus(uint8_t slave)
{
    if (slave > pins::MOTORS_COUNT)
        return;

    auto index = slave - 1;
    auto *stepper = steppers[index];

    uint8_t buffer[6];

    buffer[0] = slave;
    buffer[1] = stepper->isRunning() ? 1 : 0;
    buffer[2] = stepper->isStopping() ? 1 : 0;
    buffer[3] = stepper->getCurrentPosition();
    buffer[4] = stepper->stepsToStop();
    buffer[5] = stepper->getMaxSpeedInHz();

    protocol->sendPacket(packet::STATUS, buffer, 6);
}