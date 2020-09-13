use crate::{CmriStateMachine, MessageType, RxState};
use ruduino::legacy::serial;

/// Hardcode this for now. Only used to calculate baud rates for serial.
/// If other freqs are required then please open an issue
const CPU_FREQUENCY_HZ: u64 = 16_000_000;

/// Hardcode 64 in/64 out for now
const INPUT_BITS: u8 = 64;
const OUTPUT_BITS: u8 = 64;

/// Stores 64 input and 64 output bits as u64. This may not be as efficient
/// as using arrays of u8 on an 8-bit CPU, but hard to tell without testing
#[derive(Default)]
pub struct CmriProcessor {
    input_bits: u64,
    output_bits: u64,
    state: CmriStateMachine,
}

impl CmriProcessor {
    /// Initialise a processor attached to the given UART
    pub fn new(baud: u64) -> Self {
        let ubrr = (CPU_FREQUENCY_HZ / 16 / baud - 1) as u16;

        // Initialise the UART
        // Don't run this when running unit tests
        #[cfg(not(test))]
        serial::Serial::new(ubrr)
            .character_size(serial::CharacterSize::EightBits)
            .mode(serial::Mode::Asynchronous)
            .parity(serial::Parity::Disabled)
            .stop_bits(serial::StopBits::OneBit)
            .configure();

        // todo address filter
        Default::default()
    }

    pub fn process(&mut self) {
        use MessageType::*;
        // Read input chars while they are available
        while let Some(b) = serial::try_receive() {
            if let Ok(RxState::Complete) = self.state.process(b) {
                // got the end of a message; process its contents
                if let Some(t) = self.state.message().message_type {
                    match t {
                        Set => {
                            // copy message bits into local buffer
                        }
                        Poll => {
                            // send a response back with our local input
                            // buffer
                        }
                        _ => {}
                    }
                }
                // Break to allow program to update hardware outputs
                // with new information/pull new sensor data in before
                // next poll
                break;
            }
        }
    }

    pub fn get_bit(&self, bit: u8) -> bool {
        // Ignore overflows
        if bit > OUTPUT_BITS - 1 {
            return false;
        }

        let mask: u64 = 1 << (OUTPUT_BITS - 1 - bit);

        self.output_bits & mask != 0
    }

    pub fn get_byte(byte: u8) -> u8 {
        todo!()
    }

    pub fn set_bit(bit: u8, state: bool) {
        todo!()
    }

    pub fn set_byte(byte: u8, state: u8) {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::random;
    use std::eprintln;
    use std::format;
    use std::vec::Vec;

    fn bits(num: u64) -> Vec<bool> {
        let strbits = format!("{:064b}", num);
        strbits
            .chars()
            .map(|c| if c == '0' { false } else { true })
            .collect()
    }

    #[test]
    fn get_bit() {
        let mut p = CmriProcessor::new(9600);
        // 1111 0000 0001 0010 1010 1011 0011 0100
        // 1100 1101 0000 0000 0000 0000 1010 1010
        p.output_bits = 0xf012_ab34_cd00_00aa;

        assert_eq!(p.get_bit(0), true);
        assert_eq!(p.get_bit(1), true);
        assert_eq!(p.get_bit(4), false);
    }

    #[test]
    fn get_bit_random() {
        // Try fetching bits from five random numbers
        let mut p = CmriProcessor::new(9600);

        for _ in 0..5 {
            let number: u64 = random();
            eprintln!("Random number is: {}", number);
            eprintln!("Binary representation: {:064b}", number);
            p.output_bits = number;

            for (n, bit) in bits(number).iter().enumerate() {
                assert_eq!(p.get_bit(n as u8), *bit);
            }
        }
    }
}
