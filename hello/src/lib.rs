use windows::s;
use windows::{ Win32::Foundation::*, Win32::System::SystemServices::*, };
use windows::Win32::System::Threading::GetCurrentProcessId;

use windows::{
    Win32::Graphics::Gdi::*,
    Win32::UI::WindowsAndMessaging::*,
};

use std::mem::{transmute, MaybeUninit};
use std::io::{Result, Error, ErrorKind};

static mut PREV_WNDPROC: WNDPROC = None;

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(
    dll_module: HINSTANCE,
    call_reason: u32,
    _: *mut ())
    -> bool
{
    match call_reason {
        DLL_PROCESS_ATTACH => attach(),
        DLL_PROCESS_DETACH => detach(),
        _ => ()
    }

    true
}

fn attach() {
    unsafe {
        let handle = find_window_by_pid(GetCurrentProcessId()).unwrap();
        let result = SetWindowLongPtrW(handle, GWLP_WNDPROC, wnd_proc as isize);
        PREV_WNDPROC = transmute::<isize, WNDPROC>(result);
    };
}

fn detach() {
    unsafe {
        let handle = find_window_by_pid(GetCurrentProcessId()).unwrap();
        SetWindowLongPtrW(handle, GWLP_WNDPROC, transmute::<WNDPROC, isize>(PREV_WNDPROC));
    };
}

extern "system" fn wnd_proc(
	window: HWND,
	message: u32,
	wparam: WPARAM,
	lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match message {
            WM_PAINT => {
                let mut msg =  String::from("ZOMG!");
                let mut ps = PAINTSTRUCT::default();
                let psp = &mut ps as *mut PAINTSTRUCT;
                let rectp = &mut ps.rcPaint as *mut RECT;
                let hdc = BeginPaint(window, psp);
                let brush = CreateSolidBrush(COLORREF(0x0000F0F0));
                FillRect(hdc, &ps.rcPaint, brush);
                DrawTextA(hdc,
                    msg.as_bytes_mut(),
                    rectp,
                    DT_SINGLELINE | DT_CENTER | DT_VCENTER
                );
                EndPaint(window, &ps);
                return LRESULT(0);
            }
            WM_WINDOWPOSCHANGING => {
                let data = lparam.0 as *mut WINDOWPOS;
                let data = data.as_mut().unwrap();
                data.flags |= SWP_NOSIZE | SWP_NOMOVE;
                return LRESULT(0);
            }
            WM_NCDESTROY => {
                let result = transmute::<WNDPROC, isize>(PREV_WNDPROC);
                SetWindowLongPtrW(window, GWLP_WNDPROC, result);
                return DefWindowProcA(window, message, wparam, lparam);
            }
            _ => ()
        }
        CallWindowProcW(PREV_WNDPROC, window, message, wparam, lparam)
    }
}

pub fn find_window_by_pid(pid: u32) -> Result<HWND> {
    let mut data = EnumWindowsData {
        wanted_pid: pid,
        handle: HWND::default(),
        found: false,
    };
    unsafe {
        EnumWindows(
	        Some(enum_windows_callback),
	        LPARAM(&mut data as *mut EnumWindowsData as isize)
		);
    };
    if !data.found {
        return Err(Error::new( ErrorKind::NotFound, "Can't find the window!"));
    }
    Ok(data.handle)
}

#[derive(Default)]
struct EnumWindowsData {
    wanted_pid: u32,
    handle: HWND,
    found: bool,
}

unsafe extern "system" fn enum_windows_callback(handle: HWND, lparam: LPARAM) -> BOOL {
    let data = lparam.0 as *mut EnumWindowsData;
    let mut data = data.as_mut().unwrap();

    let mut pid = MaybeUninit::<u32>::zeroed();
    GetWindowThreadProcessId(handle, Some(pid.as_mut_ptr()));
    let pid = pid.assume_init();

    if pid == data.wanted_pid
        && GetWindow(handle, GW_OWNER).0 == 0
        && IsWindowVisible(handle).as_bool()
    {
        data.handle = handle;
        data.found = true;
        return BOOL(0);
    }

    BOOL(1)
}

#[no_mangle]
pub extern fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
