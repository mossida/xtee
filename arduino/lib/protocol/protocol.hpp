#ifndef PROTOCOL_H
#define PROTOCOL_H

#include <Arduino.h>
#include <PacketSerial.h>

namespace protocol
{
    namespace packet
    {
        const uint8_t RESET = 0x00;
        const uint8_t READY = 0x01;
        const uint8_t DATA = 0x02;
    }

    // Number of packet types we can handle
    const uint8_t PACKET_TYPES = 15;

    class HandlerBase
    {
    public:
        virtual void call(const uint8_t *data, size_t size) = 0;
        virtual ~HandlerBase() {}
    };

    template <typename T>
    class Handler : public HandlerBase
    {
        T *instance;
        void (T::*handler)(const uint8_t *, size_t);

    public:
        Handler(T *inst, void (T::*h)(const uint8_t *, size_t))
            : instance(inst), handler(h) {}

        void call(const uint8_t *data, size_t size) override
        {
            (instance->*handler)(data, size);
        }
    };

    class Protocol
    {
    private:
        PacketSerial serial;
        HandlerBase *handlers[PACKET_TYPES] = {nullptr};

        volatile bool ack = false;

        void handler(const uint8_t *data, size_t size);
        inline void handleAck() { ack = true; }

    public:
        ~Protocol()
        {
            for (int i = 0; i < PACKET_TYPES; i++)
            {
                delete handlers[i];
            }
        }

        void begin(uint32_t speed);
        void update();
        inline bool hasAcknowledged() { return ack; }

        template <typename T>
        void registerHandler(uint8_t packetId, T *instance, void (T::*handlerPtr)(const uint8_t *, size_t))
        {
            delete handlers[packetId];
            handlers[packetId] = new Handler<T>(instance, handlerPtr);
        }

        void sendPacket(uint8_t id, const uint8_t *data, size_t size);
        void announceReady();
    };
}

#endif
