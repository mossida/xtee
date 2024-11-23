#include <HX711.h>
#include <PacketSerial.h>
#include <FastAccelStepper.h>
#include <CRC16.h>

// Pin definitions
#define LOADCELL_DOUT_PIN 2
#define LOADCELL_SCK_PIN 3
#define STEPPER1_STEP_PIN 4
#define STEPPER1_DIR_PIN 5
#define STEPPER2_STEP_PIN 6
#define STEPPER2_DIR_PIN 7

// Packet IDs
#define PACKET_WEIGHT 0x01
#define PACKET_MOTOR1 0x02
#define PACKET_MOTOR2 0x03
#define PACKET_STATUS 0x04

#define SERIAL_SPEED 115200

// Global objects
FastAccelStepperEngine engine = FastAccelStepperEngine();
FastAccelStepper* stepper1 = NULL;
FastAccelStepper* stepper2 = NULL;
HX711 scale;

void packetHandler(const uint8_t* buffer, size_t size);

class Protocol {
private:
  PacketSerial serial;
  CRC16 crc = CRC16(0x8005, 0xFFFF, 0xFFFF, true, true);

  void handleMotor1Command(const uint8_t* data, size_t size) {
    if (size < 8) return;
    long steps = *((long*)data);
    long speed = *((long*)(data + 4));
    stepper1->move(steps);
    stepper1->setSpeedInHz(speed);
  }

  void handleMotor2Command(const uint8_t* data, size_t size) {
    if (size < 8) return;
    long steps = *((long*)data);
    long speed = *((long*)(data + 4));
    stepper2->move(steps);
    stepper2->setSpeedInHz(speed);
  }

  void sendPacket(uint8_t id, const uint8_t* data, size_t size) {
    uint8_t packet[size + 3];
    packet[0] = id;
    memcpy(&packet[1], data, size);

    crc.add(packet, size + 1);

    uint16_t packetCrc = crc.calc();
    packet[size + 1] = packetCrc >> 8;
    packet[size + 2] = packetCrc & 0xFF;

    serial.send(packet, size + 3);
    crc.restart();
  }

public:
  void begin(unsigned long speed) {
    serial.setPacketHandler(packetHandler);
    serial.begin(speed);
  }

  void update() {
    serial.update();
  }

  void handlePacket(const uint8_t* buffer, size_t size) {
    if (size < 4) return;  // Minimum packet size (ID + 1 byte data + CRC16)

    // Extract CRC from packet
    uint16_t receivedCrc = (buffer[size - 2] << 8) | buffer[size - 1];

    crc.add(buffer, size - 2);

    // Calculate CRC of received data
    uint16_t calculatedCrc = crc.calc();

    if (receivedCrc != calculatedCrc) return;

    uint8_t packetId = buffer[0];

    switch (packetId) {
      case PACKET_MOTOR1:
        handleMotor1Command(&buffer[1], size - 3);
        break;
      case PACKET_MOTOR2:
        handleMotor2Command(&buffer[1], size - 3);
        break;
    }

    crc.restart();
  }

  void sendWeight(float weight) {
    sendPacket(PACKET_WEIGHT, (uint8_t*)&weight, sizeof(float));
  }
};

Protocol protocol;

// Timing variables
unsigned long lastWeightUpdate = 0;

void packetHandler(const uint8_t* buffer, size_t size) {
  protocol.handlePacket(buffer, size);
}

void setup() {
  // Initialize serial communication
  protocol.begin(SERIAL_SPEED);

  // Initialize load cell
  scale.begin(LOADCELL_DOUT_PIN, LOADCELL_SCK_PIN);
  scale.set_scale(2280.f);  // Adjust this calibration factor
  scale.tare();

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
}

void loop() {
  // Update protocol
  protocol.update();

  // Read weight at fixed intervals
  if (scale.is_ready()) {
      float weight = scale.get_units(1);  // Single reading for speed
      protocol.sendWeight(weight);
  }
}
