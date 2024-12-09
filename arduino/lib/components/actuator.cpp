#include "actuator.hpp"

using namespace components::actuator;

Actuator::Actuator(protocol::Protocol *protocol)
{
    this->protocol = protocol;
}

void Actuator::begin()
{
    pinModeFast(pins::ACTUATOR_DIR, OUTPUT);
    pinModeFast(pins::ACTUATOR_PWM, OUTPUT);

    digitalWriteFast(pins::ACTUATOR_DIR, LOW);
    digitalWriteFast(pins::ACTUATOR_PWM, LOW);

    protocol->registerHandler(packet::MOVE, this, &Actuator::handleMove);
    protocol->registerHandler(packet::STOP, this, &Actuator::handleStop);
}

void Actuator::handleMove(const uint8_t *data, size_t size)
{
    if (size < 1)
        return;

    uint8_t direction = data[0];

    if (direction != 0 && direction != 1)
        return;

    if (direction == 0)
    {
        digitalWriteFast(pins::ACTUATOR_DIR, LOW);
    }
    else
    {
        digitalWriteFast(pins::ACTUATOR_DIR, HIGH);
    }

    digitalWriteFast(LED_BUILTIN, HIGH);
    digitalWriteFast(pins::ACTUATOR_PWM, HIGH);
}

void Actuator::handleStop(const uint8_t *data, size_t size)
{
    if (size != 0)
        return;

    digitalWriteFast(LED_BUILTIN, LOW);

    digitalWriteFast(pins::ACTUATOR_PWM, LOW);
    digitalWriteFast(pins::ACTUATOR_DIR, LOW);
}
