#include "actuator.hpp"

using namespace components::actuator;

Actuator::Actuator(protocol::Protocol *protocol)
{
    this->protocol = protocol;
}

void Actuator::begin()
{
    pinMode(pins::ACTUATOR_DIR, OUTPUT);
    pinMode(pins::ACTUATOR_PWM, OUTPUT);

    digitalWrite(pins::ACTUATOR_DIR, LOW);
    digitalWrite(pins::ACTUATOR_PWM, LOW);

    protocol->registerHandler(packet::MOVE, this, &Actuator::handleMove);
    protocol->registerHandler(packet::STOP, this, &Actuator::handleStop);
}

void Actuator::handleMove(const uint8_t *data, size_t size)
{
    digitalWrite(LED_BUILTIN, HIGH);

    if (size < 1)
        return;

    uint8_t direction = data[0];

    if (direction != 0 && direction != 1)
        return;

    digitalWrite(pins::ACTUATOR_PWM, HIGH);
    digitalWrite(pins::ACTUATOR_DIR, direction);
}

void Actuator::handleStop(const uint8_t *data, size_t size)
{
    digitalWrite(LED_BUILTIN, LOW);

    if (size != 0)
        return;

    digitalWrite(pins::ACTUATOR_PWM, LOW);
    digitalWrite(pins::ACTUATOR_DIR, LOW);
}
