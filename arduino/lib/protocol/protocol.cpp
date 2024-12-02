#include "protocol.hpp"

using namespace protocol;

void Protocol::handler(const uint8_t *data, size_t size)
{
    if (size < 2)
        return;

    auto receivedCrc = data[size - 1];

    if (receivedCrc != crc::calculate(data, size - 1))
        return;

    auto dataSize = size - 2;
    auto packetId = data[0];

    if (packetId < 0 || packetId >= 256)
        return;

    if (packetId == packet::READY)
        return handleAck();

    if (handlers[packetId])
    {
        handlers[packetId]->call(data + 1, dataSize);
    }
}

void Protocol::begin(uint32_t speed)
{
    serial.begin(speed);
    serial.setPacketHandler([](const void *sender, const uint8_t *buffer, size_t size)
                            {
                               Protocol *protocol = (Protocol *)sender;
                               protocol->handler(buffer, size); });

    sendPacket(packet::READY, nullptr, 0);
}

void Protocol::update()
{
    serial.update();
}

void Protocol::buildPacket(uint8_t *buffer, uint8_t id, const uint8_t *data, size_t size)
{
    buffer[0] = id;

    if (data && size > 0)
        memcpy(&buffer[1], data, size);

    auto packetCRC = crc::calculate(buffer, size + 1);
    buffer[size + 1] = packetCRC;
}

void Protocol::sendPacket(uint8_t id, const uint8_t *data, size_t size)
{
    if (!ack && id != packet::READY)
        return;

    uint8_t buffer[size + 2];
    buildPacket(buffer, id, data, size);
    serial.send(buffer, size + 2);
}

void Protocol::handleAck()
{
    ack = true;
}