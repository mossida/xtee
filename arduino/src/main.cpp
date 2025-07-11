#include <PacketSerial.h>

#define ENABLE_SCALE 0
#define ENABLE_MOTORS 1
#define ENABLE_ACTUATOR 0

#include "protocol.hpp"

#define SERIAL_SPEED 115200

protocol::Protocol pt;

#if ENABLE_SCALE
#include "scale.hpp"
components::scale::Scale sc(&pt);
#endif

#if ENABLE_ACTUATOR
#include "actuator.hpp"
components::actuator::Actuator at(&pt);
#endif

#if ENABLE_MOTORS
#include "motor.hpp"
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
