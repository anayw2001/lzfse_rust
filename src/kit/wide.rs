// Fixed constant. Max copy width. Do not change.
pub const WIDE: usize = 32;

mod private {
  pub trait Sealed {}
}

pub trait Width: private::Sealed {
    const POWER: u8;
    const _CHECK: () = assert!(Self::POWER <= 5, "Exponent is higher than supported!");
    const WIDTH: usize = 1usize << Self::POWER;
}

#[derive(Copy, Clone, Debug)]
pub struct W00;

impl private::Sealed for W00 {}
impl Width for W00 {
    const POWER: u8 = 4;
}

#[derive(Copy, Clone, Debug)]
pub struct W08;

impl private::Sealed for W08 {}
impl Width for W08 {
    const POWER: u8 = 3;
}

#[derive(Copy, Clone, Debug)]
pub struct W16;

impl private::Sealed for W16 {}
impl Width for W16 {
    const POWER: u8 = 4;
}

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub struct Wide;

impl private::Sealed for Wide {}
impl Width for Wide {
    const POWER: u8 = 5;
}
