
#[derive(Copy, Clone, Default)]
#[cfg_attr(any(feature = "fmt", test), derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Mode {
    #[default]
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
