#include <PacketSerial.h>

#define ENABLE_SCALE 1
#define ENABLE_MOTORS 0
#define ENABLE_ACTUATOR 1

#if ENABLE_SCALE
#include <HX711.h>
#endif

#if ENABLE_MOTORS
#include <FastAccelStepper.h>
#endif

#include "protocol.hpp"
#include "components.hpp"

#define SERIAL_SPEED 230400

protocol::Protocol pt;

#if ENABLE_SCALE
components::scale::Scale sc(&pt);
#endif

#if ENABLE_ACTUATOR
components::actuator::Actuator at(&pt);
#endif

#if ENABLE_MOTORS
components::motor::Engine mt(&pt);
#endif

void setup()
{
  pinModeFast(LED_BUILTIN, OUTPUT);

  // Initialize serial communication
  pt.begin(SERIAL_SPEED);

#if ENABLE_SCALE
  sc.begin();
#endif

#if ENABLE_ACTUATOR
  at.begin();
#endif

#if ENABLE_MOTORS
  mt.begin();
#endif
}

void loop()
{
#if ENABLE_SCALE
  sc.update();
#endif

  pt.update();
}
