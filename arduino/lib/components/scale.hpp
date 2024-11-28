#ifndef SCALE_H
#define SCALE_H

#include <HX711.h>

#include "protocol.hpp"
#include "pins.hpp"

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
            Scale(protocol::Protocol *protocol);

            void begin();
            void update();

        private:
            protocol::Protocol *protocol;
            HX711 sensor;
        };
    }
}

#endif
