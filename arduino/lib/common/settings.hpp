#ifndef SETTINGS_H
#define SETTINGS_H

#include <Arduino.h>

namespace settings
{
    static const uint16_t MOTOR_STEPS = 800;
    static const uint32_t MOTOR_ROTATIONS_LIMIT = (INT32_MAX / MOTOR_STEPS) * 10;
}

#endif