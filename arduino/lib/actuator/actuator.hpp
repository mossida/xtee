#ifndef ACTUATOR_H
#define ACTUATOR_H

#include <Arduino.h>
#include <digitalWriteFast.h>
#include "common.hpp"
#include "protocol.hpp"

namespace components
{
    namespace actuator
    {
        namespace packet
        {
            const uint8_t MOVE = 0x0E;
            const uint8_t STOP = 0x0F;

            typedef struct __attribute__((packed))
            {
                bool direction;
            } MOVE_DATA;
        }

        class Actuator
        {
        public:
            explicit Actuator(protocol::Protocol *protocol);

            void begin();

            void handleMove(const uint8_t *data, size_t size);
            void handleStop(const uint8_t *data, size_t size);

        private:
            protocol::Protocol *protocol;
        };
    }
}

#endif
