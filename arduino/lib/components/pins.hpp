#ifndef PINS_H
#define PINS_H

#include <Arduino.h>

namespace pins
{
    const uint8_t SCALE_DOUT = 2;
    const uint8_t SCALE_SCK = 3;

    const uint8_t ACTUATOR_DIR = 5;
    const uint8_t ACTUATOR_PWM = 4;
}

#endif