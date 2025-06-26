#include "scale.hpp"

using namespace components::scale;

Scale::Scale(protocol::Protocol *protocol) : protocol(protocol)
{
}

void Scale::begin()
{
    sensor.begin();

    // TODO: Needs calibration?
}

void Scale::update()
{
    if (protocol->hasAcknowledged() && sensor.available())
    {
        auto weight = sensor.read();
        protocol->sendPacket(packet::WEIGHT, (uint8_t *)&weight, sizeof(weight));
    }
}