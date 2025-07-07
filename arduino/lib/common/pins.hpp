#ifndef PINS_H
#define PINS_H

#include <Arduino.h>

namespace pins
{
    struct Motor
    {
        const uint8_t step;
        const uint8_t dir;
        const uint8_t enable;
    };

    static const uint8_t ACTUATOR_DIR = 8;
    static const uint8_t ACTUATOR_PWM = 7;

    static const Motor MOTOR_1 = {3, 2, 4};
    static const Motor MOTOR_2 = {6, 5, 7};

    static const Motor *MOTORS[] = {&MOTOR_1, &MOTOR_2};
    static const size_t MOTORS_COUNT = 2;
}

#endif