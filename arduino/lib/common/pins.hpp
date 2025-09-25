#ifndef PINS_H
#define PINS_H

#include <Arduino.h>

namespace pins
{
    struct Motor
    {
        const uint8_t step; // Use gpio pins
        const uint8_t dir; // Use D pins
        const uint8_t enable; // Use D pins
    };

    static const uint8_t ACTUATOR_DIR = 3;
    static const uint8_t ACTUATOR_PWM = 2;


    static const Motor MOTOR_1 = {6, 2, 4};
    static const Motor MOTOR_2 = {9, 5, 7};
    // static const Motor MOTOR_1 = {9, 2, 4};
    // static const Motor MOTOR_2 = {6, 5, 7};

    static const Motor *MOTORS[] = {&MOTOR_1, &MOTOR_2};
    static const size_t MOTORS_COUNT = 2;
}

#endif