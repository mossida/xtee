export interface SerialPortType {
  vid: number;
  pid: number;
  serial_number: string | null;
  manufacturer: string | null;
  product: string | null;
}

export interface SerialPort {
  port_name: string;
  port_type: {
    UsbPort: SerialPortType;
  };
}
