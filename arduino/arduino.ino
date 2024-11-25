#include <HX711.h>
#include <PacketSerial.h>
#include <FastAccelStepper.h>
#include <avr/pgmspace.h>

// Pin definitions
#define LOADCELL_DOUT_PIN 2
#define LOADCELL_SCK_PIN 3
#define STEPPER1_STEP_PIN 4
#define STEPPER1_DIR_PIN 5
#define STEPPER2_STEP_PIN 6
#define STEPPER2_DIR_PIN 7

#define ACTUATOR_PWM 4
#define ACTUATOR_DIR 5

// Packet IDs
#define PACKET_READY 0x01            // READY_ID
#define PACKET_DATA 0x02             // DATA_ID
#define PACKET_HEALTH_REQUEST 0x03   // HEALTH_REQUEST_ID
#define PACKET_HEALTH_RESPONSE 0x04  // HEALTH_RESPONSE_ID
#define PACKET_MOTOR_COMMAND 0x05    // MOTOR_COMMAND_ID
#define PACKET_MOTOR_STOP 0x06       // MOTOR_STOP_ID
#define PACKET_ACTUATOR_MOVE 0x07    // ACTUATOR_MOVE_ID
#define PACKET_ACTUATOR_STOP 0x08    // ACTUATOR_STOP_ID
#define PACKET_TARE_CELL 0x09        // TARE_CELL_ID
#define PACKET_TARE_SUCCESS 0x0a     // TARE_SUCCESS_ID

#define SERIAL_SPEED 115200

// Global objects
FastAccelStepperEngine engine = FastAccelStepperEngine();
FastAccelStepper *stepper1 = NULL;
FastAccelStepper *stepper2 = NULL;
HX711 scale;

void packetHandler(const uint8_t *buffer, size_t size);

class PacketCRC {
public:
  PacketCRC() {}

  // Calculate the CRC for a given data array
  uint8_t calculate(const uint8_t *arr, size_t len) {
    uint8_t crc = 0;  // Initial CRC value

    for (size_t i = 0; i < len; i++) {
      crc = csTable[crc ^ arr[i]];
    }

    return crc;
  }

private:
  // Precomputed CRC table for polynomial 0x9B
  const uint8_t csTable[256] = {
    0x00, 0x9B, 0xAD, 0x36, 0xC1, 0x5A, 0x6C, 0xF7, 0x19, 0x82, 0xB4, 0x2F, 0xD8, 0x43, 0x75, 0xEE, 0x32, 0xA9, 0x9F, 0x04, 0xF3, 0x68, 0x5E, 0xC5, 0x2B, 0xB0, 0x86, 0x1D, 0xEA, 0x71, 0x47, 0xDC, 0x64, 0xFF, 0xC9, 0x52, 0xA5, 0x3E, 0x08, 0x93, 0x7D, 0xE6, 0xD0, 0x4B, 0xBC, 0x27, 0x11, 0x8A, 0x56, 0xCD, 0xFB, 0x60, 0x97, 0x0C, 0x3A, 0xA1, 0x4F, 0xD4, 0xE2, 0x79, 0x8E, 0x15, 0x23, 0xB8, 0xC8, 0x53, 0x65, 0xFE, 0x09, 0x92, 0xA4, 0x3F, 0xD1, 0x4A, 0x7C, 0xE7, 0x10, 0x8B, 0xBD, 0x26, 0xFA, 0x61, 0x57, 0xCC, 0x3B, 0xA0, 0x96, 0x0D, 0xE3, 0x78, 0x4E, 0xD5, 0x22, 0xB9, 0x8F, 0x14, 0xAC, 0x37, 0x01, 0x9A, 0x6D, 0xF6, 0xC0, 0x5B, 0xB5, 0x2E, 0x18, 0x83, 0x74, 0xEF, 0xD9, 0x42, 0x9E, 0x05, 0x33, 0xA8, 0x5F, 0xC4, 0xF2, 0x69, 0x87, 0x1C, 0x2A, 0xB1, 0x46, 0xDD, 0xEB, 0x70, 0x0B, 0x90, 0xA6, 0x3D, 0xCA, 0x51, 0x67, 0xFC, 0x12, 0x89, 0xBF, 0x24, 0xD3, 0x48, 0x7E, 0xE5, 0x39, 0xA2, 0x94, 0x0F, 0xF8, 0x63, 0x55, 0xCE, 0x20, 0xBB, 0x8D, 0x16, 0xE1, 0x7A, 0x4C, 0xD7, 0x6F, 0xF4, 0xC2, 0x59, 0xAE, 0x35, 0x03, 0x98, 0x76, 0xED, 0xDB, 0x40, 0xB7, 0x2C, 0x1A, 0x81, 0x5D, 0xC6, 0xF0, 0x6B, 0x9C, 0x07, 0x31, 0xAA, 0x44, 0xDF, 0xE9, 0x72, 0x85, 0x1E, 0x28, 0xB3, 0xC3, 0x58, 0x6E, 0xF5, 0x02, 0x99, 0xAF, 0x34, 0xDA, 0x41, 0x77, 0xEC, 0x1B, 0x80, 0xB6, 0x2D, 0xF1, 0x6A, 0x5C, 0xC7, 0x30, 0xAB, 0x9D, 0x06, 0xE8, 0x73, 0x45, 0xDE, 0x29, 0xB2, 0x84, 0x1F, 0xA7, 0x3C, 0x0A, 0x91, 0x66, 0xFD, 0xCB, 0x50, 0xBE, 0x25, 0x13, 0x88, 0x7F, 0xE4, 0xD2, 0x49, 0x95, 0x0E, 0x38, 0xA3, 0x54, 0xCF, 0xF9, 0x62, 0x8C, 0x17, 0x21, 0xBA, 0x4D, 0xD6, 0xE0, 0x7B
  };
};

class Protocol {
private:
  PacketSerial serial;
  PacketCRC crc;

  boolean transmissionReady = false;

  void handleActuatorMoveCommand(const uint8_t *data, size_t size) {
    digitalWrite(LED_BUILTIN, HIGH);
    
    if (size < 1)
      return;

    uint8_t direction = data[0];

    if (direction != 0 && direction != 1)
      return;

    digitalWrite(ACTUATOR_PWM, HIGH);
    digitalWrite(ACTUATOR_DIR, direction);
  }

  void handleActuatorStopCommand(const uint8_t *data, size_t size) {
    digitalWrite(LED_BUILTIN, LOW);
    
    if (size != 0)
      return;

    digitalWrite(ACTUATOR_PWM, LOW);
    digitalWrite(ACTUATOR_DIR, LOW);
  }

  void handleMotor1Command(const uint8_t *data, size_t size) {
    if (size < 8)
      return;
    long steps = *((long *)data);
    long speed = *((long *)(data + 4));
    stepper1->move(steps);
    stepper1->setSpeedInHz(speed);
  }

  void handleMotor2Command(const uint8_t *data, size_t size) {
    if (size < 8)
      return;
    long steps = *((long *)data);
    long speed = *((long *)(data + 4));
    stepper2->move(steps);
    stepper2->setSpeedInHz(speed);
  }

  void sendPacket(uint8_t id, const uint8_t *data, size_t size) {
    if (!transmissionReady && id != PACKET_READY)
      return;

    uint8_t packet[size + 2];

    packet[0] = id;

    if (data && size > 0)
      memcpy(&packet[1], data, size);

    uint8_t packetCRC = crc.calculate(packet, size + 1);
    packet[size + 1] = packetCRC;

    serial.send(packet, size + 2);
  }

public:
  void begin(unsigned long speed) {
    serial.begin(speed);
    serial.setPacketHandler(packetHandler);

    sendPacket(PACKET_READY, nullptr, 0);
  }

  void update() {
    serial.update();
  }

  void handlePacket(const uint8_t *buffer, size_t size) {
    digitalWrite(LED_BUILTIN, LOW);

    if (size < 2)
      return;

    // Extract CRC from packet
    uint8_t receivedCrc = buffer[size - 1];

    if (receivedCrc != crc.calculate(buffer, size - 1))
      return;

    uint8_t packetId = buffer[0];

    switch (packetId) {
      case PACKET_READY:
        transmissionReady = true;
        break;
      case PACKET_ACTUATOR_MOVE:
        handleActuatorMoveCommand(&buffer[1], size - 2);
        break;
      case PACKET_ACTUATOR_STOP:
        handleActuatorStopCommand(&buffer[1], size - 2);
        break;
      case PACKET_MOTOR_COMMAND:
        if (size < 7)
          return;  // Check minimum size for motor command
        uint8_t slave = buffer[1];
        if (slave == 1) {
          handleMotor1Command(&buffer[2], size - 4);
        } else if (slave == 2) {
          handleMotor2Command(&buffer[2], size - 4);
        }
        break;
      case PACKET_MOTOR_STOP:
        if (size < 2)
          return;
        uint8_t stopSlave = buffer[1];
        if (stopSlave == 1 && stepper1) {
          stepper1->stopMove();
        } else if (stopSlave == 2 && stepper2) {
          stepper2->stopMove();
        }
        break;
    }
  }

  void sendWeight(long weight) {
    sendPacket(PACKET_DATA, (uint8_t *)&weight, sizeof(long));
  }
};

Protocol protocol;

void packetHandler(const uint8_t *buffer, size_t size) {
  protocol.handlePacket(buffer, size);
}

void setup() {
  // Initialize serial communication
  protocol.begin(SERIAL_SPEED);

  // Initialize load cell
  scale.begin(LOADCELL_DOUT_PIN, LOADCELL_SCK_PIN);

  // Initialize stepper motors
  engine.init();
  stepper1 = engine.stepperConnectToPin(STEPPER1_STEP_PIN);
  stepper2 = engine.stepperConnectToPin(STEPPER2_STEP_PIN);

  if (stepper1) {
    stepper1->setDirectionPin(STEPPER1_DIR_PIN);
    stepper1->setAcceleration(10000);
  }

  if (stepper2) {
    stepper2->setDirectionPin(STEPPER2_DIR_PIN);
    stepper2->setAcceleration(10000);
  }

  pinMode(ACTUATOR_DIR, OUTPUT);
  pinMode(ACTUATOR_PWM, OUTPUT);
  pinMode(LED_BUILTIN, OUTPUT);

  digitalWrite(ACTUATOR_DIR, LOW);
  digitalWrite(ACTUATOR_PWM, LOW);
}

void loop() {
  if (scale.is_ready()) {
    protocol.sendWeight(scale.read());
  }

  protocol.update();
}
