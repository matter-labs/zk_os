#[repr(usize)]
pub enum Registers {
    Zero = 0,
    Ra,
    Sp,
    Gp,
    Tp,
    T0,
    T1,
    T2,
    S0,
    S1,
    A0, /* 10 */
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    S2,
    S3,
    S4, /* 20 */
    S5,
    S6,
    S7,
    S8,
    S9,
    S10,
    S11,
    T3,
    T4,
    T5, /* 30 */
    T6,
}

#[inline(always)]
pub const fn gp(r: Registers) -> usize {
    r as usize
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapFrame {
    pub regs: [u32; 32], // 0..128
                         // pub satp:   u32,       // 128..132
                         // pub pc:     u32,       // 132..136
                         // pub qm:     u32,       // 136..140
                         // pub pid:    u32,       // 140..144
                         // pub mode:   u32,       // 144..148
}

impl TrapFrame {
    pub const fn new() -> Self {
        TrapFrame {
            regs: [0; 32],
            // satp:   0,
            // pc:     0,
            // qm:     1,
            // pid:    0,
            // mode:   0,
        }
    }
}

/// Dumps the registers of a given trap frame. This is NOT the
/// current CPU registers!
pub fn dump_registers(frame: *const TrapFrame) {
    use crate::{print, println};

    print!("   ");
    for i in 1..32 {
        if i % 4 == 0 {
            println!();
            print!("   ");
        }
        print!("x{:2}:{:08x}   ", i, unsafe { (*frame).regs[i] });
    }
    println!();
}
