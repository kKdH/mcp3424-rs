
#[derive(Copy, Clone)]
#[cfg_attr(feature = "fmt", derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Mode {
    Continuous,
    OneShot,
}

impl Mode {
    pub const fn mask(&self) -> u8 {
        match self {
            Mode::Continuous => 1,
            Mode::OneShot => 0,
        }
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Continuous
    }
}
