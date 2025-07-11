#include "protocol.hpp"
#include <digitalWriteFast.h>

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

    if (packetId == packet::RESET)
    {
        ack = false;
        return;
    }

    if (packetId == packet::READY)
        return handleAck();

    if (handlers[packetId])
    {
        handlers[packetId]->call(data + 1, dataSize);
    }
}

void Protocol::begin(uint32_t speed)
{
    Serial.begin(speed);

    // Only wait for Serial on boards with native USB
#ifdef USBCON
    while (!Serial) {;}
#else
    delay(100);
#endif

    serial.setStream(&Serial);
    serial.setPacketHandler([](const void *sender, const uint8_t *buffer, size_t size)
                            {
                               Protocol *protocol = (Protocol *)sender;
                               protocol->handler(buffer, size); });
}

void Protocol::update()
{
    serial.update();

    static unsigned long lastAnnouncement = 0;
    if (!hasAcknowledged() && millis() - lastAnnouncement > 1000)
    {
        announceReady();
        lastAnnouncement = millis();
    }
}

void Protocol::sendPacket(uint8_t id, const uint8_t *data, size_t size)
{
    if (!ack && id != packet::READY)
        return;

    uint8_t buffer[size + 2];

    buffer[0] = id;

    if (data && size > 0)
        memcpy(&buffer[1], data, size);

    buffer[size + 1] = crc::calculate(buffer, size + 1);

    serial.send(buffer, size + 2);
}

void Protocol::announceReady()
{
    sendPacket(packet::READY, nullptr, 0);
}