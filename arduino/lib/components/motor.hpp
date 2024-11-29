#ifndef MOTOR_H
#define MOTOR_H

#include <Arduino.h>
#include <FastAccelStepper.h>

#include "protocol.hpp"
#include "pins.hpp"

namespace components
{
    namespace motor

    {
        namespace packet
        {
            const uint8_t MOVE = 0x03;
            const uint8_t SETTINGS = 0x04;
            const uint8_t REPORT_STATUS = 0x05;
            const uint8_t STATUS = 0x06;
            const uint8_t STOP = 0x07;
        }

        class Engine
        {
        public:
            explicit Engine(protocol::Protocol *protocol);

            void begin();

            void handleMove(const uint8_t *data, size_t size);
            void handleSettings(const uint8_t *data, size_t size);
            void handleReportStatus(const uint8_t *data, size_t size);
            void handleStop(const uint8_t *data, size_t size);

        private:
            protocol::Protocol *protocol;
            FastAccelStepperEngine engine;
            FastAccelStepper *steppers[pins::MOTORS_COUNT];
        };
    }
}

#endif