#ifndef MOTOR_H
#define MOTOR_H

#include <Arduino.h>
#include <FastAccelStepper.h>
#include <digitalWriteFast.h>

#include "common.hpp"
#include "protocol.hpp"

namespace components
{
    namespace motor

    {
        namespace packet
        {
            const uint8_t MOVE = 0x03;
            const uint8_t KEEP = 0x04;

            const uint8_t SET_SPEED = 0x05;
            const uint8_t SET_ACCELERATION = 0x06;
            const uint8_t SET_OUTPUTS = 0x07;

            const uint8_t RECOGNITION = 0x08;
            const uint8_t REPORT_STATUS = 0x09;
            const uint8_t STATUS = 0x0A;
            const uint8_t STOP = 0x0B;
        }

        class Engine
        {
        public:
            explicit Engine(protocol::Protocol *protocol) : protocol(protocol) {}

            void begin();

            void handleMove(const uint8_t *data, size_t size);
            void handleKeep(const uint8_t *data, size_t size);
            void handleSetSpeed(const uint8_t *data, size_t size);
            void handleSetAcceleration(const uint8_t *data, size_t size);
            void handleSetOutputs(const uint8_t *data, size_t size);
            void handleReportStatus(const uint8_t *data, size_t size);
            void handleStop(const uint8_t *data, size_t size);

        private:
            bool is_initialized = false;
            protocol::Protocol *protocol;
            FastAccelStepperEngine engine;
            FastAccelStepper *steppers[pins::MOTORS_COUNT] = {nullptr};

            void sendStatus(uint8_t slave);
            void sendRecognition(uint8_t slave);
        };
    }
}

#endif