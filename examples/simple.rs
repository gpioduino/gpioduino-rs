use gpioduino::{Conn, DigitalValue, PinMode};

fn main() {
    let mut conn = Conn::new("COM3").expect("failed to open port");

    // conn.dtr(true).expect("failed to set DTR"); // only needed for certain boards like leonardo

    conn.pin_mode(13, PinMode::Output)
        .expect("failed to set pin 13 to OUTPUT");
    conn.digital_write(13, DigitalValue::High)
        .expect("failed to digital write HIGH to pin 13");

    match conn.digital_read(13) {
        Ok(val) => println!("Value: {:?}", val),
        Err(err) => println!("Error: {:?}", err),
    }

    conn.digital_write(13, DigitalValue::Low)
        .expect("failed to digital write LOW to pin 13");
}
