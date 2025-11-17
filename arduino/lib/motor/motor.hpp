#ifndef MOTOR_H
#define MOTOR_H

#include <Arduino.h>
#include <FastAccelStepper.h>
#include <cstddef>
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

            typedef struct __attribute__((packed))
            {
                uint8_t slave;
                bool direction;
                bool deferred;
            } KEEP_DATA;

            typedef struct __attribute__((packed))
            {
                uint8_t slave;
                bool direction;
                uint32_t rotations;
                bool deferred;
            } MOVE_DATA;

            typedef struct __attribute__((packed))
            {
                uint8_t slave;
                bool gentle;
                bool deferred;
            } STOP_DATA;

            typedef struct __attribute__((packed))
            {
                bool gentle;
            } STOP_ALL_DATA;

            typedef struct __attribute__((packed))
            {
                uint8_t slave;
                bool apply;
                uint32_t speed;
            } SET_SPEED_DATA;

            typedef struct __attribute__((packed))
            {
                uint8_t slave;
                bool apply;
                uint32_t acceleration;
            } SET_ACCELERATION_DATA;

            typedef struct __attribute__((packed))
            {
                uint8_t slave;
                bool outputs;
            } SET_OUTPUTS_DATA;

            typedef struct __attribute__((packed))
            {
                uint8_t slave;
                uint32_t max_speed;
            } RECOGNITION_DATA;

            typedef struct __attribute__((packed))
            {
                uint8_t slave;
                bool running;
                bool stopping;
                bool outputs;
                int32_t position;
                uint32_t remaining;
            } STATUS_DATA;

            typedef struct __attribute__((packed))
            {
                uint8_t slave;
            } REPORT_STATUS_DATA;

            struct QueuedCommand
            {
                uint8_t type;
                uint8_t slave;
                union
                {
                    MOVE_DATA move;
                    KEEP_DATA keep;
                    STOP_DATA stop;
                } data;
            };
        }

        class Engine
        {
        public:
            explicit Engine(protocol::Protocol *protocol) : protocol(protocol), engine() {}

            void begin();

            void handleMove(const uint8_t *data, size_t size);
            void handleKeep(const uint8_t *data, size_t size);
            void handleSetSpeed(const uint8_t *data, size_t size);
            void handleSetAcceleration(const uint8_t *data, size_t size);
            void handleSetOutputs(const uint8_t *data, size_t size);
            void handleReportStatus(const uint8_t *data, size_t size);
            void handleStop(const uint8_t *data, size_t size);
            void handleSync(const uint8_t *data, size_t size);

        private:
            bool is_initialized = false;

            protocol::Protocol *protocol;

            FastAccelStepperEngine engine;
            FastAccelStepper *steppers[pins::MOTORS_COUNT] = {nullptr};

            packet::QueuedCommand commandQueue[pins::MOTORS_COUNT];
            size_t commandQueueSize = 0;

            void sendStatus(uint8_t slave);
            void sendRecognition(uint8_t slave);

            void executeQueue();
            void executeMove(const packet::MOVE_DATA &data);
            void executeKeep(const packet::KEEP_DATA &data);
            void executeStop(const packet::STOP_DATA &data);
        };
    }
}

#endif
