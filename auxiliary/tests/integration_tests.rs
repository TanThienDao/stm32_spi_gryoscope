use auxiliary::{detect_gyroscope, GyroVariant};
use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::OutputPin;

// Mock SPI and CS pin for testing
struct MockSpi {
    response: Vec<u8>,
    call_count: usize,
}

impl MockSpi {
    fn new(response: Vec<u8>) -> Self {
        Self {
            response,
            call_count: 0,
        }
    }
}

// Implement the Transfer trait for MockSpi
impl Transfer<u8> for MockSpi {
    type Error = ();

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        // Simulate SPI response by filling buffer with predefined response
        for (i, byte) in self.response.iter().enumerate() {
            if i < words.len() {
                words[i] = *byte;
            }
        }
        self.call_count += 1;
        Ok(words)
    }
}

// Mock CS pin
struct MockCs {
    low_called: bool,
    high_called: bool,
}

impl MockCs {
    fn new() -> Self {
        Self {
            low_called: false,
            high_called: false,
        }
    }
}

impl OutputPin for MockCs {
    type Error = ();

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.low_called = true;
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.high_called = true;
        Ok(())
    }
}

#[test]
/// Test the detection of I3G4250D gyroscope
fn test_detect_i3g4250d() {
    let mut spi = MockSpi::new(vec![0x00, 0xD3]); // Simulate WHO_AM_I response for I3G4250D
    let mut cs = MockCs::new();
    let result = detect_gyroscope(&mut spi, &mut cs);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), GyroVariant::I3g4250d);
    assert!(cs.low_called);
    assert!(cs.high_called);
}

#[test]
/// Test the detection of L3GD20 gyroscope
fn test_detect_l3gd20() {
    let mut spi = MockSpi::new(vec![0x00, 0xD4]);
    let mut cs = MockCs::new();
    let result = detect_gyroscope(&mut spi, &mut cs);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), GyroVariant::L3gd20);
}

#[test]
/// Test the detection of L3GD20H gyroscope
fn test_detect_l3gd20h() {
    let mut spi = MockSpi::new(vec![0x00, 0xD7]);
    let mut cs = MockCs::new();
    let result = detect_gyroscope(&mut spi, &mut cs);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), GyroVariant::L3gd20h);
}

#[test]
/// Test the detection of an unknown gyroscope
fn test_detect_unknown_gyroscope() {
    let mut spi = MockSpi::new(vec![0x00, 0xFF]);
    let mut cs = MockCs::new();
    let result = detect_gyroscope(&mut spi, &mut cs);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), GyroVariant::Unknown(0xFF));
}

#[test]
/// Test the control of the CS pin
fn test_cs_pin_control() {
    let mut spi = MockSpi::new(vec![0x00, 0xD3]);
    let mut cs = MockCs::new();
    let _ = detect_gyroscope(&mut spi, &mut cs);
    assert!(cs.low_called, "CS pin should be pulled low");
    assert!(cs.high_called, "CS pin should be pulled high");
}