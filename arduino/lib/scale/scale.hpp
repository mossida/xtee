#ifndef SCALE_H
#define SCALE_H

#include <Adafruit_NAU7802.h>

#include "common.hpp"
#include "protocol.hpp"

namespace components
{
    namespace scale
    {
        namespace packet
        {
            const uint8_t WEIGHT = 0x02;
        }

        class Scale
        {
        public:
            explicit Scale(protocol::Protocol *protocol);

            void begin();
            void update();

        private:
            protocol::Protocol *protocol;
            Adafruit_NAU7802 sensor;
        };
    }
}

#endif
