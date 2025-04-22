//! Process management syscalls

use crate::mm::address::StepByOne;
use crate::config::PAGE_SIZE;
use crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, get_system_call, insert_framed_area, current_user_token, delete_framed_area};
use crate::mm::{translated_va_to_pa, MapPermission, PageTable, PhysAddr, VirtAddr};
use crate::timer::{get_time_us};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let ts = translated_va_to_pa(current_user_token(), ts as usize) as *mut TimeVal;
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        }
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 => {
            let page_table = PageTable::from_token(current_user_token());

            let vpn = VirtAddr::from(id).floor();
            let offset = VirtAddr::from(id).page_offset();
            let pte = match page_table.translate(vpn) {
                Some(pte) => {
                    if pte.is_user() && pte.readable() {
                        pte
                    } else { 
                        return -1;
                    }
                }
                None => return -1,
            };
            let ppn= pte.ppn();
            let pa = PhysAddr::from(ppn);
            let ptr = (usize::from(pa) | offset) as  *const u8;
            unsafe {
                (*ptr) as isize
            }
        },
        1 => {
            let page_table = PageTable::from_token(current_user_token());

            let vpn = VirtAddr::from(id).floor();

            let offset = VirtAddr::from(id).page_offset();
            let pte = match page_table.translate(vpn) {
                Some(pte) => {
                    if pte.is_user() && pte.writable() {
                        pte
                    } else {
                        return -1;
                    }
                }
                None => return -1,
            };
            let ppn= pte.ppn();
            let pa = PhysAddr::from(ppn);
            let ptr = (usize::from(pa) | offset) as *mut u8;
            unsafe {
                *ptr = data as u8;
            }
            0
        },
        2 => {
            get_system_call(id) as isize
        },
        _ => unreachable!(),
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(start + len);
    
    if start_va.page_offset() != 0 || port & 0x7 == 0 || port & !0x7 != 0 {
        return -1;
    }

    let page_table = PageTable::from_token(current_user_token());
    let mut vpn = start_va.floor();
    for _ in 0..(len + PAGE_SIZE - 1) / PAGE_SIZE {
        match page_table.translate(vpn) {
            Some(pte) => {
                if pte.is_valid() {
                    return -1;
                }
            }
            None => {}
        }
        vpn.step();
    }
    
    let permission = MapPermission::from_bits_truncate((port << 1)as u8);
    insert_framed_area(start_va, end_va, permission | MapPermission::U);
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap!");
    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(start + len);
    if start_va.page_offset() != 0 {
        return -1
    }
    let page_table = PageTable::from_token(current_user_token());
    let mut vpn = start_va.floor();
    for _ in 0..(len + PAGE_SIZE - 1) / PAGE_SIZE {
        match page_table.translate(vpn) {
            Some(pte) => {
                if !pte.is_valid() {
                    return -1;
                }
            }
            None => {
                return -1;
            }
        }
        vpn.step();
    }

    delete_framed_area(start_va, end_va);
    0
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
