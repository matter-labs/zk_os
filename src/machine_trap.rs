use core::hint::unreachable_unchecked;

use crate::helper_reg_utils::*;
use crate::trap_frame::MachineTrapFrame;
use crate::utils::*;

use riscv::register::{mstatus::MPP, satp::Mode};

#[inline(never)]
fn machine_mode_handle_unaligned_load(
    trap_frame: &mut MachineTrapFrame,
    instr: u32,
    epc: usize,
) -> (usize, bool) {
    // Simulator and circuit disable unaligned loads (but still can load individual u8/u16/u32 without crossing memory boundary),
    // so we should only expect cases when we cross the boudnary

    let rd = get_rd(instr);
    if rd == 0 {
        unsafe { riscv::asm::wfi() };
    }

    // now depending on how many bytes we need to load we proceed
    let mut imm = ITypeOpcode::imm(instr);
    sign_extend(&mut imm, 12);
    let rs1 = ITypeOpcode::rs1(instr);
    let rs1: u32 = trap_frame.registers[rs1 as usize];
    let physical_address = rs1.wrapping_add(imm);

    let aligned_address = physical_address & !3;
    let unalignment = physical_address & 3;

    let funct3 = ITypeOpcode::funct3(instr);

    let bytes_to_read = match funct3 {
        0 | 4 => 1,
        1 | 5 => 2,
        2 => 4,
        _ => return (0, true), // invalid instruction
    };

    // now just match over everything

    // no translation here
    let value: u32 = match (unalignment, bytes_to_read) {
        (1, 2) => {
            // single write
            let value_low =
                unsafe { core::ptr::from_exposed_addr::<u32>(aligned_address as usize).read() };

            value_low >> 8
        }
        (1, 4) | (2, 4) | (3, 4) | (3, 2) => {
            let (next_address, overflow) = aligned_address.overflowing_add(4);
            if overflow {
                unsafe { riscv::asm::wfi() };
            }

            let value_low =
                unsafe { core::ptr::from_exposed_addr::<u32>(aligned_address as usize).read() };
            let value_high =
                unsafe { core::ptr::from_exposed_addr::<u32>(next_address as usize).read() };
            // properly shift to get value. Sign/zero extend below takes care of cleaning up top bytes if needed
            let shift = unalignment * 8;

            (value_low >> shift) | (value_high << (32 - shift))
        }
        _ => unsafe {
            riscv::asm::wfi();

            unreachable_unchecked()
        },
    };

    let ret_val = match funct3 {
        1 => sign_extend_16(value),
        2 => value,
        5 => zero_extend_16(value),
        _ => unsafe {
            riscv::asm::wfi();

            unreachable_unchecked()
        },
    };

    trap_frame.registers[rd as usize] = ret_val;

    // return to mepc + 4
    (epc.wrapping_add(4), false)
}

#[inline(never)]
fn machine_mode_handle_unaligned_store(
    trap_frame: &mut MachineTrapFrame,
    instr: u32,
    epc: usize,
) -> (usize, bool) {
    // Same - we only handle u16/u32 unaligned stores

    let mut imm = STypeOpcode::imm(instr);
    sign_extend(&mut imm, 12);

    let rs1 = STypeOpcode::rs1(instr);
    let rs1: u32 = trap_frame.registers[rs1 as usize];
    let physical_address = rs1.wrapping_add(imm);

    let funct3 = STypeOpcode::funct3(instr);
    let bytes_to_write = match funct3 {
        a @ 0 | a @ 1 | a @ 2 => 1 << a,
        _ => return (0, true), // invalid instruction
    };

    let aligned_address = physical_address & !3;
    let unalignment = physical_address & 3;

    let rs2 = STypeOpcode::rs2(instr);
    let rs2: u32 = trap_frame.registers[rs2 as usize];
    let value_to_write = rs2;

    match (unalignment, bytes_to_write) {
        (1, 2) => {
            // single read and write
            let existing_value_low =
                unsafe { core::ptr::from_exposed_addr::<u32>(aligned_address as usize).read() };
            let new_value = existing_value_low & 0xff0000ff & ((value_to_write & 0x0000ffff) << 8);
            unsafe {
                core::ptr::from_exposed_addr_mut::<u32>(aligned_address as usize).write(new_value)
            };
        }
        (1, 4) | (2, 4) | (3, 4) | (3, 2) => {
            let (next_address, overflow) = aligned_address.overflowing_add(4);
            if overflow {
                unsafe { riscv::asm::wfi() };
            }

            let existing_value_low =
                unsafe { core::ptr::from_exposed_addr::<u32>(aligned_address as usize).read() };
            let existing_value_high =
                unsafe { core::ptr::from_exposed_addr::<u32>(next_address as usize).read() };

            let value_mask = match bytes_to_write {
                2 => 0x0000ffffu32,
                4 => 0xffffffffu32,
                _ => unsafe { unreachable_unchecked() },
            };
            let masked_value = value_to_write & value_mask;

            let (mask_existing_low, mask_existing_high) = match (unalignment, bytes_to_write) {
                (1, 4) => (0x000000ffu32, 0xffffff00u32),
                (2, 4) => (0x0000ffffu32, 0xffff0000u32),
                (3, 4) => (0x00ffffffu32, 0xff000000u32),
                (3, 2) => (0x00ffffffu32, 0xffffff00u32),
                _ => unsafe { unreachable_unchecked() },
            };

            let shift = unalignment * 8;
            let new_low = (existing_value_low & mask_existing_low) | (masked_value << shift);
            let new_high =
                (existing_value_high & mask_existing_high) | (masked_value >> (32 - shift));

            unsafe {
                core::ptr::from_exposed_addr_mut::<u32>(aligned_address as usize).write(new_low)
            };
            unsafe {
                core::ptr::from_exposed_addr_mut::<u32>(next_address as usize).write(new_high)
            };
        }
        _ => unsafe {
            riscv::asm::wfi();

            unreachable_unchecked()
        },
    };

    // return to mepc + 4
    (epc.wrapping_add(4), false)
}

#[link_section = ".trap.rust"]
#[export_name = "MachineExceptionHandler"]
fn custom_machine_exception_handler(trap_frame: &mut MachineTrapFrame) -> usize {
    let cause = riscv::register::mcause::read();
    let status = riscv::register::mstatus::read();
    let previous_mode = status.mpp();
    let cause_num = cause.code();
    let epc = riscv::register::mepc::read();
    let satp = riscv::register::satp::read();

    // fast track for misaligned memory access
    match cause_num {
        0 | 4 | 6 => {
            if previous_mode == MPP::Machine || satp.mode() == Mode::Bare {
                // we do not need a translation, but we also have an opcode value in the TVAL
                let instr = riscv::register::mtval::read();
                let instr = instr as u32;

                //// we can also do by dereference
                // let mepc = core::ptr::from_exposed_addr::<u32>(epc);
                // let instr = unsafe { mepc.read() };

                let opcode = get_opcode(instr);

                if opcode == 0b0000011 {
                    // LOAD
                    let (new_pc, invalid_instruction) =
                        machine_mode_handle_unaligned_load(trap_frame, instr, epc);
                    if invalid_instruction {
                        unsafe { riscv::asm::wfi() }
                    } else {
                        return new_pc;
                    }
                } else if opcode == 0b0100011 {
                    // STORE
                    let (new_pc, invalid_instruction) =
                        machine_mode_handle_unaligned_store(trap_frame, instr, epc);
                    if invalid_instruction {
                        unsafe { riscv::asm::wfi() }
                    } else {
                        return new_pc;
                    }
                } else {
                    unsafe { riscv::asm::wfi() }
                }
            } else {
                // need translation
                unsafe { riscv::asm::wfi() }
            }
        }
        _ => {}
    }

    crate::rust_abort();
}
