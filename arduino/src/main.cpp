#include <HX711.h>
#include <PacketSerial.h>
#include <FastAccelStepper.h>

#include "protocol.hpp"
#include "components.hpp"

#define ENABLE_SCALE 1
#define ENABLE_MOTORS 0
#define ENABLE_ACTUATOR 1

#define SERIAL_SPEED 115200

protocol::Protocol pt;

#if ENABLE_SCALE
components::scale::Scale sc(&pt);
#endif

#if ENABLE_ACTUATOR
components::actuator::Actuator at(&pt);
#endif
void setup()
{
  pinMode(LED_BUILTIN, OUTPUT);

  // Initialize serial communication
  pt.begin(SERIAL_SPEED);

#if ENABLE_SCALE
  sc.begin();
#endif

#if ENABLE_ACTUATOR
  at.begin();
#endif
}

void loop()
{
#if ENABLE_SCALE
  sc.update();
#endif

  pt.update();
}
