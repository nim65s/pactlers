#![no_std]

use bincode::{Decode, Encode};

pub const HEADER: [u8; 4] = [0xFF, 0xFF, 0xFD, 0];
pub const N_ADCS: usize = 5;

#[derive(Encode, Decode, Debug, Eq, PartialEq, Copy, Clone)]
pub struct Cmd {
    pub select: u8,
    pub volume: u16,
}
