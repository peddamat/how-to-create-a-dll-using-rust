use dll_syringe::{Syringe, process::*};
use windows::{Win32::{UI::WindowsAndMessaging::{GetWindowThreadProcessId, FindWindowA}, Foundation::HWND}, core::PCSTR, s};
use std::{thread, time, borrow::BorrowMut, mem::MaybeUninit};


fn main() {
    // find target process by .exe
    let target_process =
	    OwnedProcess::find_first_by_name("ttermpro.exe").expect("Couldn't find process, exiting!");

    // find target process window handle
    // let pid = find_pid_by_hwnd(HWND(0x00401B32));
    // let target_process = OwnedProcess::from_pid(pid).unwrap();

    // find target process by window title
    // let pid = find_pid_by_title(s!("Untitled - Notepad"));
    // let target_process = OwnedProcess::from_pid(pid).unwrap();

   let dll_path = {
        if OwnedProcess::is_x64(&target_process).unwrap() {
            "target\\x86_64-pc-windows-msvc\\debug\\hello.dll"
        } else {
            "target\\i686-pc-windows-msvc\\debug\\hello.dll"
        }
    };

    // create a new syringe for the target process
    let syringe = Syringe::for_process(target_process);

    let injected_payload = syringe.inject(dll_path).expect("Architecture mismatch!");
    println!("DLL injected successfully!");

    // do something else
    let ten_millis = time::Duration::from_secs(10);

    println!("Sleeping for 30 secs...");
    thread::sleep(ten_millis);

    // eject the payload from the target (optional)
    match syringe.eject(injected_payload) {
        Ok(_) => println!("hello.dll ejected successfully."),
        Err(_) => println!("Couldn't find process, assuming it's closed and exiting gracefully!")
    }
}

fn find_pid_by_hwnd(hwnd: HWND) -> u32 {
    let mut pid = MaybeUninit::<u32>::zeroed();
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(pid.as_mut_ptr()));
        return pid.assume_init();
    };
}

pub fn find_pid_by_title(title: PCSTR) -> u32 {
    let hwnd = unsafe {
        FindWindowA(None, title)
    };

    return find_pid_by_hwnd(hwnd);
}