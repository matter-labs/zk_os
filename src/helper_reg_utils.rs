use super::*;
use crate::utils::*;

#[must_use]
#[inline(always)]
pub const fn get_opcode(src: u32) -> u32 {
    src & 0b01111111 // opcode is always lowest 7 bits
}

#[must_use]
#[inline(always)]
pub const fn get_rd(src: u32) -> u32 {
    (src >> 7) & 0b00011111
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ITypeOpcode;

impl ITypeOpcode {
    #[must_use]
    #[inline(always)]
    pub const fn rs1(src: u32) -> u32 {
        get_bits_and_align_right(src, 15, 5)
    }

    #[must_use]
    #[inline(always)]
    pub const fn funct3(src: u32) -> u32 {
        get_bits_and_align_right(src, 12, 3)
    }

    #[must_use]
    #[inline(always)]
    pub const fn imm(src: u32) -> u32 {
        get_bits_and_align_right(src, 20, 12)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct STypeOpcode;

impl STypeOpcode {
    #[must_use]
    #[inline(always)]
    pub const fn rs1(src: u32) -> u32 {
        get_bits_and_align_right(src, 15, 5)
    }

    #[must_use]
    #[inline(always)]
    pub const fn rs2(src: u32) -> u32 {
        get_bits_and_align_right(src, 20, 5)
    }

    #[must_use]
    #[inline(always)]
    pub const fn funct3(src: u32) -> u32 {
        get_bits_and_align_right(src, 12, 3)
    }

    #[must_use]
    #[inline(always)]
    pub const fn imm(src: u32) -> u32 {
        get_bits_and_align_right(src, 7, 5) | 
        get_bits_and_shift_right(src, 25, 7, 25 - 5)
    }
}