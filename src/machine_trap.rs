use core::hint::unreachable_unchecked;

use crate::{trap_frame::MachineTrapFrame};
use crate::helper_reg_utils::*;
use crate::utils::*;

use riscv::register::{mstatus::MPP, satp::Mode};

#[inline(never)]
fn handle_unaligned_load(
    trap_frame: &mut MachineTrapFrame,
    instr: u32,
    epc: usize,
) -> (usize, bool) {
    // now depending on how many bytes we need to load we proceed
    let funct3 = ITypeOpcode::funct3(instr);

    let bytes_to_read = match funct3 {
        0 | 4 => 1,
        1 | 5 => 2,
        2 => 4,
        _ => {return (0, true)} // invalid instruction
    };

    let rd = get_rd(instr);
    let mut imm = ITypeOpcode::imm(instr);
    sign_extend(&mut imm, 12);
    let rs1 = ITypeOpcode::rs1(instr);
    let rs1: u32 = trap_frame.registers[rs1 as usize];
    // no translation here
    let physical_address = rs1.wrapping_add(imm) as usize;
    let unalignement = physical_address & 0x3;
    let aligned_address = core::ptr::from_exposed_addr::<u32>(physical_address & !0x3);

    let unalignment_bits = unalignement*8;
    let value: u32 = match (unalignement, bytes_to_read) {
        (1, 2) | (2, 2) | (1, 1) | (2, 1) | (3, 1) => {
            let value_low = unsafe { aligned_address.read() };

            value_low >> unalignment_bits
        },
        (3, 2) | (1, 4) | (2, 4) | (3, 4) => {
            let value_low = unsafe { aligned_address.read() };
            let value_high = unsafe { aligned_address.add(1).read() };
            // properly shift to get value

            (value_low >> unalignment_bits) | (value_high << (32 - unalignment_bits))
        
        },
        _ => { unsafe {unreachable_unchecked()} }
    };

    let ret_val = match funct3 {
        0 => sign_extend_8(value),
        1 => sign_extend_16(value),
        2 => value,
        4 => zero_extend_8(value),
        5 => zero_extend_16(value),
        _ => unsafe {unreachable_unchecked()}
    };

    trap_frame.registers[rd as usize] = ret_val;

    // return to mepc + 4
    (epc.wrapping_add(4), false)
}

#[inline(never)]
fn handle_unaligned_store(
    trap_frame: &mut MachineTrapFrame,
    instr: u32,
    epc: usize,
) -> (usize, bool) {
    let funct3 = STypeOpcode::funct3(instr);
    let bytes_to_write = match funct3 {
        a @ 0 | a @ 1 | a @ 2 => 1 << a,
        _ => {return (0, true)} // invalid instruction
    };

    let mut imm = STypeOpcode::imm(instr);
    sign_extend(&mut imm, 12);

    let rs1 = STypeOpcode::rs1(instr);
    let rs1: u32 = trap_frame.registers[rs1 as usize];
    let physical_address = rs1.wrapping_add(imm) as usize;

    let unalignement = physical_address & 0x3;
    let aligned_address = core::ptr::from_exposed_addr_mut::<u32>(physical_address & !0x3);

    let rs2 = STypeOpcode::rs2(instr);
    let rs2: u32 = trap_frame.registers[rs2 as usize];
    let value_to_write = rs2;

    let unalignment_bits = unalignement*8;
    match (unalignement, bytes_to_write) {
        (1, 2) | (2, 2) | (1, 1) | (2, 1) | (3, 1) => {
            // we only need 1 access
            let existing_value_low = unsafe { aligned_address.read() };
            let mask_for_existing_low = (1 << unalignment_bits) - 1;
            let new_low = (existing_value_low & mask_for_existing_low) | (value_to_write << unalignment_bits);
            unsafe { aligned_address.write(new_low) };
        },
        (3, 2) | (1, 4) | (2, 4) | (3, 4) => {
            let existing_value_low = unsafe { aligned_address.read() };
            let existing_value_high = unsafe { aligned_address.add(1).read() };
            // properly shift to get value
            let mask_for_existing_low = (1 << unalignment_bits) - 1;
            let new_low = (existing_value_low & mask_for_existing_low) | (value_to_write << unalignment_bits);
            let mask_for_existing_high = !mask_for_existing_low;
            let new_high = (existing_value_high & mask_for_existing_high) | (value_to_write >> (32 - unalignment_bits));
            unsafe { aligned_address.write(new_low) };
            unsafe { aligned_address.add(1).write(new_high) };        
        },
        _ => { unsafe {unreachable_unchecked()} }
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
                // we do not need a translation
                let mepc = core::ptr::from_exposed_addr::<u32>(epc);
                let instr = unsafe { mepc.read() };
                let opcode = get_opcode(instr);
                if opcode == 0b0000011 {
                    // LOAD
                    let (new_pc, invalid_instruction) = handle_unaligned_load(trap_frame, instr, epc);
                    if invalid_instruction {
                        unsafe { riscv::asm::wfi() }
                    } else {
                        return new_pc;
                    }
                } else if opcode == 0b0100011 {
                    // STORE
                    let (new_pc, invalid_instruction) = handle_unaligned_store(trap_frame, instr, epc);
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
            
        },
        _ => {}
    }

    let tval = riscv::register::mtval::read();
    let hart = riscv::register::mhartid::read();

    crate::rust_abort();

    // use crate::println;

	// // The cause contains the type of trap (sync, async) as well as the cause
	// // number. So, here we narrow down just the cause number.
	// let cause_num = cause & 0xfff;
	// let mut return_pc = epc;
    // match cause_num {
    //     2 => unsafe {
    //         // Illegal instruction
    //         // println!("Illegal instruction CPU#{} -> 0x{:08x}: 0x{:08x}\n", hart, epc, tval);

    //         crate::rust_abort();

    //         // delete_process((*frame).pid as u16);
    //         // let frame = schedule();
    //         // schedule_next_context_switch(1);
    //         // rust_switch_to_user(frame);
    //     },
    //     3 => {
    //         // breakpoint
    //         // println!("BKPT\n\n");
    //         return_pc += 4;
    //     },
    //     // 7 => unsafe {
    //     // 	println!("Error with pid {}, at PC 0x{:08x}, mepc 0x{:08x}", (*frame).pid, (*frame).pc, epc);
    //     // },
    //     8 | 9 | 11 => unsafe {
    //         crate::rust_abort();
    //     },
    //     // Page faults
    //     12 => unsafe {
    //         // Instruction page fault
    //         // println!("Instruction page fault CPU#{} -> 0x{:08x}: 0x{:08x}", hart, epc, tval);
    //     },
    //     13 => unsafe {
    //         // Load page fault
    //         // println!("Load page fault CPU#{} -> 0x{:08x}: 0x{:08x}", hart, epc, tval);
    //     },
    //     15 => unsafe {
    //         // Store page fault
    //         // println!("Store page fault CPU#{} -> 0x{:08x}: 0x{:08x}", hart, epc, tval);
    //     },
    //     _ => {
    //         // panic!(
    //         //         "Unhandled sync trap {}. CPU#{} -> 0x{:08x}: 0x{:08x}\n",
    //         //         cause_num, hart, epc, tval
    //         // );
    //     }
    // };
    // 
	// // Finally, return the updated program counter
	// return_pc
}