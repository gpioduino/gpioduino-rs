const MAX_BYTES: usize = 150;

const SETTINGS: serialport::SerialPortSettings = serialport::SerialPortSettings {
    baud_rate: 9600,
    data_bits: serialport::DataBits::Eight,
    parity: serialport::Parity::None,
    stop_bits: serialport::StopBits::One,
    flow_control: serialport::FlowControl::None,
    timeout: std::time::Duration::from_millis(1000),
};

pub struct Conn {
    reader: Box<dyn serialport::SerialPort>,
    writer: Box<dyn serialport::SerialPort>,
}

impl Conn {
    pub fn new<T: AsRef<std::ffi::OsStr> + ?Sized>(port: &T) -> Result<Conn, serialport::Error> {
        let reader = serialport::open_with_settings(port, &SETTINGS)?;
        let writer = reader.try_clone()?;
        Ok(Conn { reader, writer })
    }

    pub fn dtr(&mut self, dtr: bool) -> Result<(), Error> {
        self.writer.write_data_terminal_ready(dtr)?;
        Ok(())
    }

    pub fn pin_mode(&mut self, pin: i8, mode: PinMode) -> Result<(), Error> {
        let cmd = match mode {
            PinMode::Input => format!("PM {} I;", pin),
            PinMode::Output => format!("PM {} O;", pin),
        };

        self.execute_command(cmd)?;
        Ok(())
    }

    pub fn digital_write(&mut self, pin: i8, val: DigitalValue) -> Result<(), Error> {
        let cmd = match val {
            DigitalValue::Low => format!("DW {} L;", pin),
            DigitalValue::High => format!("DW {} H;", pin),
        };

        self.execute_command(cmd)?;
        Ok(())
    }

    pub fn digital_read(&mut self, pin: i8) -> Result<DigitalValue, Error> {
        let cmd = format!("DR {};", pin);
        let val = self.execute_command(cmd)?;

        match val.as_str() {
            "L" => Ok(DigitalValue::Low),
            "H" => Ok(DigitalValue::High),
            _ => Err(Error::UnkownDigitalValue),
        }
    }

    pub fn analog_write(&mut self, pin: i8, val: u8) -> Result<(), Error> {
        let cmd = format!("AW {} {};", pin, val);
        self.execute_command(cmd)?;
        Ok(())
    }

    pub fn analog_read(&mut self, pin: i8) -> Result<u8, Error> {
        let cmd = format!("AR {};", pin);
        let val = self.execute_command(cmd)?;

        let val: u8 = val.parse()?;
        Ok(val)
    }

    pub fn echo(&mut self, val: &str) -> Result<String, Error> {
        let cmd = format!("ECHO {};", val);
        self.execute_command(cmd)
    }

    pub fn raw(&mut self, val: &str) -> Result<String, Error> {
        self.execute_command(String::from(val))
    }

    fn execute_command(&mut self, cmd: String) -> Result<String, Error> {
        self.writer.write_all(cmd.as_bytes())?;

        let mut buffer: Vec<u8> = vec![0; MAX_BYTES];
        let _ = self.reader.read(&mut buffer)?;

        let res = String::from_utf8(buffer)?;
        let res = res.trim_matches(char::from(0)).trim_matches(';');

        let mut parts = res.split_ascii_whitespace();

        let res = parts.next().unwrap_or("ERROR");
        let arg = parts.next().unwrap_or("");

        match res {
            "ACK" => Ok(String::from(arg)),
            "ERROR" => {
                let e: Error = arg.parse().unwrap_or(Error::UnknownError);
                Err(e)
            }
            _ => Err(Error::UnknownError),
        }
    }
}

#[derive(Debug)]
pub enum PinMode {
    Input,
    Output,
}

#[derive(Debug)]
pub enum DigitalValue {
    Low,
    High,
}

impl std::str::FromStr for DigitalValue {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "L" => Ok(DigitalValue::Low),
            "H" => Ok(DigitalValue::High),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Timeout,
    NoCommand,
    UnknownCommand,
    UnkonwnPinMode,
    UnkownDigitalValue,
    Other(Box<dyn std::error::Error>),
    UnknownError,
}

/// errfrom! implements std::convert::From for Error
/// by simply returning Error::Other with the boxed error
macro_rules! errfrom {
    (
      $($impl_type: path), +
    ) => {
      $(
          impl std::convert::From<$impl_type> for Error {
              fn from(err: $impl_type) -> Self {
                  Error::Other(Box::from(err))
              }
          }
      )+
  };
}

errfrom!(
    std::io::Error,
    std::string::FromUtf8Error,
    std::num::ParseIntError,
    serialport::Error
);

impl std::str::FromStr for Error {
    type Err = Self;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TIMEOUT" => Ok(Error::Timeout),
            "NO_COMMAND" => Ok(Error::NoCommand),
            "UNKNOWN_COMMAND" => Ok(Error::UnknownCommand),
            "UNKNOWN_MODE" => Ok(Error::UnkonwnPinMode),
            "UNKNOWN_DIGITAL_VALUE" => Ok(Error::UnkownDigitalValue),
            _ => Ok(Error::UnknownError),
        }
    }
}
