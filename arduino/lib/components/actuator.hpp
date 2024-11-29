#ifndef ACTUATOR_H
#define ACTUATOR_H

#include <Arduino.h>
#include <digitalWriteFast.h>

#include "protocol.hpp"
#include "pins.hpp"

namespace components
{
    namespace actuator
    {
        namespace packet
        {
            const uint8_t MOVE = 0x08;
            const uint8_t STOP = 0x09;
        }

        class Actuator
        {
        public:
            Actuator(protocol::Protocol *protocol);

            void begin();

            void handleMove(const uint8_t *data, size_t size);
            void handleStop(const uint8_t *data, size_t size);

        private:
            protocol::Protocol *protocol;
        };
    }
}

#endif