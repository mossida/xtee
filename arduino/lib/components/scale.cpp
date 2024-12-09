#include "scale.hpp"

using namespace components::scale;

Scale::Scale(protocol::Protocol *protocol) : protocol(protocol)
{
}

void Scale::begin()
{
    sensor.begin(pins::SCALE_DOUT, pins::SCALE_SCK);
}

void Scale::update()
{
    if (protocol->hasAcknowledged() && sensor.is_ready())
    {
        auto weight = sensor.read();
        protocol->sendPacket(packet::WEIGHT, (uint8_t *)&weight, sizeof(weight));
    }
}