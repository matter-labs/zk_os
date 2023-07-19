use crate::cpu::*;
use crate::println;

#[no_mangle]
extern "C" fn m_trap(
    epc: usize,
    tval: usize,
    cause: usize,
    hart: usize,
    _status: usize,
    frame: *mut TrapFrame,
) -> usize {
    // We're going to handle all traps in machine mode. RISC-V lets
    // us delegate to supervisor mode, but switching out SATP (virtual memory)
    // gets hairy.
    let is_async = {
        if cause >> 31 & 1 == 1 {
            true
        } else {
            false
        }
    };
    // The cause contains the type of trap (sync, async) as well as the cause
    // number. So, here we narrow down just the cause number.
    let cause_num = cause & 0xfff;
    let mut return_pc = epc;
    if is_async {
        // Asynchronous trap
        match cause_num {
            3 => {
                // We will use this to awaken our other CPUs so they can process
                // processes.
                println!("Machine software interrupt CPU #{}", hart);
            }
            7 => {
                println!("Timeout on CPU #{}", hart);
            }
            11 => {
                println!("Interrupt on CPU #{}", hart);
            }
            _ => {
                panic!("Unhandled async trap CPU#{} -> {}\n", hart, cause_num);
            }
        }
    } else {
        // Synchronous trap
        match cause_num {
            2 => unsafe {
                // Illegal instruction
                println!(
                    "Illegal instruction CPU#{} -> 0x{:08x}: 0x{:08x}\n",
                    hart, epc, tval
                );

                // delete_process((*frame).pid as u16);
                // let frame = schedule();
                // schedule_next_context_switch(1);
                // rust_switch_to_user(frame);
            },
            3 => {
                // breakpoint
                println!("BKPT\n\n");
                return_pc += 2;
            }
            // 7 => unsafe {
            // 	println!("Error with pid {}, at PC 0x{:08x}, mepc 0x{:08x}", (*frame).pid, (*frame).pc, epc);
            // },
            8 | 9 | 11 => unsafe {},
            // Page faults
            12 => unsafe {
                // Instruction page fault
                println!(
                    "Instruction page fault CPU#{} -> 0x{:08x}: 0x{:08x}",
                    hart, epc, tval
                );
            },
            13 => unsafe {
                // Load page fault
                println!(
                    "Load page fault CPU#{} -> 0x{:08x}: 0x{:08x}",
                    hart, epc, tval
                );
            },
            15 => unsafe {
                // Store page fault
                println!(
                    "Store page fault CPU#{} -> 0x{:08x}: 0x{:08x}",
                    hart, epc, tval
                );
            },
            _ => {
                panic!(
                    "Unhandled sync trap {}. CPU#{} -> 0x{:08x}: 0x{:08x}\n",
                    cause_num, hart, epc, tval
                );
            }
        }
    };
    // Finally, return the updated program counter
    return_pc
}
