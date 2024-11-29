#include "motor.hpp"

using namespace components::motor;

Engine::Engine(protocol::Protocol *protocol) : protocol(protocol)
{
    for (size_t i = 1; i <= pins::MOTORS_COUNT; i++)
    {
        steppers[i - 1] = nullptr;
    }
}

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

    protocol->registerHandler(packet::SETTINGS, this, &Engine::handleSettings);
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