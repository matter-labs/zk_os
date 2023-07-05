#![no_std]
#![feature(panic_info_message)]
#![feature(fn_align)]
#![feature(const_mut_refs)]
#![feature(strict_provenance)]
#![no_main]
#![no_builtins]

extern "C" {
    // Boundaries of the .bss section
    static mut _ebss: u32;
    static mut _sbss: u32;

    // Boundaries of the .data section
    static mut _edata: u32;
    static mut _sdata: u32;

    // Initial values of the .data section (stored in Flash)
    static _sidata: u32;
}

core::arch::global_asm!(include_str!("asm/asm.S"));
// core::arch::global_asm!(include_str!("asm/trap.S"));
// core::arch::global_asm!(include_str!("asm/boot.S"));
// core::arch::global_asm!(include_str!("asm/mem.S"));

pub mod trap;
pub mod quasi_uart;
pub mod cpu;
pub mod trap_frame;
pub mod machine_trap;
pub mod helper_reg_utils;
pub mod utils;

use riscv::register::{mcause as xcause, mtvec as xtvec, mtvec::TrapMode as xTrapMode};
use riscv_rt::__INTERRUPTS;

use self::trap_frame::MachineTrapFrame;

// ///////////////////////////////////
// / RUST MACROS
// ///////////////////////////////////
#[macro_export]
macro_rules! print
{
	($($args:tt)+) => ({
		use core::fmt::Write;
		let _ = write!(crate::quasi_uart::QuasiUART::new(crate::quasi_uart::QUASI_UART_ADDRESS), $($args)+);
	});
}
#[macro_export]
macro_rules! println
{
	() => ({
		crate::print!("\r\n")
	});
	($fmt:expr) => ({
		crate::print!(concat!($fmt, "\r\n"))
	});
	($fmt:expr, $($args:tt)+) => ({
		crate::print!(concat!($fmt, "\r\n"), $($args)+)
	});
}


#[no_mangle]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
	// print!("Aborting: ");
	// if let Some(p) = info.location() {
	// 	println!(
	// 				"line {}, file {}: {}",
	// 				p.line(),
	// 				p.file(),
	// 				info.message().unwrap()
	// 	);
	// }
	// else {
	// 	println!("no information available.");
	// }
	rust_abort();
}

#[no_mangle]
pub extern "Rust"
fn rust_abort() -> ! {
	loop {
		continue;
	}
	// loop {
	// 	unsafe {
	// 		asm!("wfi");
	// 	}
	// }
}		

// #[no_mangle]
// #[repr(align(4))]
// extern "Rust"
// fn kmain() {
// 	println!("kmain");
// }

// #[no_mangle]
// #[repr(align(4))]
// extern "Rust"
// fn kinit() {
// 	// we do not have traps setup yet, so don't use unaligned access please!
// }

// #[no_mangle]
// #[repr(align(4))]
// extern "Rust"
// fn kinit_hart() {
// 	println!("kinit_hart");
// }

use riscv_rt::entry;

#[entry]
#[inline(never)]
fn main() -> ! {
	let mut pinger = quasi_uart::QuasiUART::new(quasi_uart::QUASI_UART_ADDRESS);
	use core::fmt::Write;
	let _ = pinger.write_str("Hello from kernel");
	// pinger.write_word(0x0000000a);
	// println!("Hello from kernel");
	// unsafe { GLOBAL_MACHINE_UART.write_word(0x000a) };
	loop { }
}

use riscv_rt::pre_init;

#[pre_init]
unsafe fn machine_pre_init() {
	// extern "C" {
    //     fn _machine_start_trap();
    // }
    // xtvec::write(_machine_start_trap as *const () as usize, xTrapMode::Direct);
}

#[export_name = "_mp_hook"]
pub extern "Rust" fn mp_hook(hartid: usize) -> bool {
    match hartid {
        0 => true,
        _ => loop {
            unsafe { riscv::asm::wfi() }
        },
    }
}

#[export_name = "_setup_interrupts"]
pub unsafe extern "Rust" fn custom_setup_interrupts() {
	extern "C" {
        fn _machine_start_trap();
    }

    // xtvec::write(_machine_start_trap as *const () as usize, xTrapMode::Direct);
}

#[link_section = ".trap.rust"]
#[export_name = "_machine_start_trap_rust"]
pub extern "C" fn machine_start_trap_rust(trap_frame: *mut MachineTrapFrame) -> usize {
    extern "C" {
        fn MachineExceptionHandler(trap_frame: &mut MachineTrapFrame) -> usize;
		fn DefaultHandler();
    }

    unsafe {
        let cause = xcause::read();

        if cause.is_exception() {
            MachineExceptionHandler(&mut *trap_frame)
        } else {
            if cause.code() < __INTERRUPTS.len() {
                let h = &__INTERRUPTS[cause.code()];
                if h.reserved == 0 {
                    DefaultHandler();
                } else {
                    (h.handler)();
                }
            } else {
                DefaultHandler();
            }

			riscv::register::mepc::read()
        }
    }
}