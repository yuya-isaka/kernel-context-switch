#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

use core::arch::{asm, global_asm};

global_asm!(
    r#"
.section ".text.boot.mykernel"
.global boot_mykernel
boot_mykernel:
    la sp, __stack
    j  kernel_main
    "#
);

unsafe fn sbi_call(a0: i64, a1: i64, fid: i64, eid: i64) {
    asm!(
        "ecall",
        in("a0") a0, in("a1") a1, in("a6") fid, in("a7") eid
    );
}

fn putchar(ch: u8) {
    unsafe {
        sbi_call(ch as i64, 0, 0, 0x01);
    }
}

fn print(s: &str) {
    for ch in s.as_bytes() {
        putchar(*ch);
    }
}

#[no_mangle]
fn kernel_main() -> ! {
    print("Hello, world!\n");

    unsafe {
        TH_A.init(th_a_entry as usize);
        TH_B.init(th_b_entry as usize);

        th_yield();
    }

    panic!("booted!");
}

// コンテキストスイッチ ↓ ==========================================================

global_asm!(
    r#"
.align 8
.global switch_context
switch_context:
    addi sp, sp, -13 * 8
    sd ra, 8 * 0(sp)
    sd s0,  8 * 1(sp)
    sd s1, 8 * 2(sp)
    sd s2, 8 * 3(sp)
    sd s3, 8 * 4(sp)
    sd s4, 8 * 5(sp)
    sd s5, 8 * 6(sp)
    sd s6, 8 * 7(sp)
    sd s7, 8 * 8(sp)
    sd s8, 8 * 9(sp)
    sd s9, 8 * 10(sp)
    sd s10, 8 * 11(sp)
    sd s11, 8 * 12(sp)
    sd sp, (a0)
    ld sp, (a1)
    ld ra, 8 * 0(sp)
    ld s0, 8 * 1(sp)
    ld s1, 8 * 2(sp)
    ld s2, 8 * 3(sp)
    ld s3, 8 * 4(sp)
    ld s4, 8 * 5(sp)
    ld s5, 8 * 6(sp)
    ld s6, 8 * 7(sp)
    ld s7, 8 * 8(sp)
    ld s8, 8 * 9(sp)
    ld s9, 8 * 10(sp)
    ld s10, 8 * 11(sp)
    ld s11, 8 * 12(sp)
    addi sp, sp, 13 * 8
    ret
    "#
);

use core::ptr::null_mut;

static mut TH_A: Th = Th::new();
static mut TH_B: Th = Th::new();
static mut CU: *mut Th = null_mut();

#[repr(align(8))]
#[derive(Debug, Clone, Copy)]
struct Th {
    sp: u64,
    stack: [u8; 8192],
}

impl Th {
    const fn new() -> Self {
        Self {
            sp: 0,
            stack: [0; 8192],
        }
    }

    fn init(&mut self, entry: usize) {
        use core::mem::size_of_val;
        let sp = self
            .stack
            .as_mut_ptr()
            .wrapping_add(size_of_val(&self.stack)) as *mut u64;

        unsafe {
            *sp.sub(1) = 0; // t0
            *sp.sub(2) = 0; // t1
            *sp.sub(3) = 0; // t2
            *sp.sub(4) = 0; // t3
            *sp.sub(5) = 0; // t4
            *sp.sub(6) = 0; // t5
            *sp.sub(7) = 0; // t6
            *sp.sub(8) = 0; // a0
            *sp.sub(9) = 0; // a1
            *sp.sub(10) = 0; // a2
            *sp.sub(11) = 0; // a3
            *sp.sub(12) = 0; // a4
            *sp.sub(13) = entry as u64; // return address

            self.sp = sp.sub(13) as u64;
        }
    }
}

extern "C" {
    fn switch_context(prev_sp: *mut u64, next_sp: *const u64);
}

fn th_a_entry() {
    loop {
        print("A ");
        unsafe {
            for _ in 0..1000000 {
                asm!("nop");
            }
            th_yield();
        }
    }
}

fn th_b_entry() {
    loop {
        print("B ");
        unsafe {
            for _ in 0..1000000 {
                asm!("nop");
            }
            th_yield();
        }
    }
}

#[allow(static_mut_refs)]
unsafe fn th_yield() {
    let next_proc = if CU == &mut TH_A {
        &mut TH_B
    } else {
        &mut TH_A
    };

    let prev_proc = CU;
    CU = next_proc;

    switch_context(&mut (*prev_proc).sp, &next_proc.sp);
}
