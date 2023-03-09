use windows::s;
use windows::{ Win32::Foundation::*, Win32::System::SystemServices::*, };
use windows::Win32::System::Threading::GetCurrentProcessId;

use windows::{
    Win32::Graphics::Gdi::*,
    Win32::UI::WindowsAndMessaging::*,
};

use std::collections::HashMap;

use std::mem::{transmute, MaybeUninit};
use std::io::{Result, Error, ErrorKind};

use log::{LevelFilter, info, error};

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
        _ => true
    }
}

fn attach() -> bool {
    unsafe {
        simple_logging::log_to_file("C:\\Users\\me\\source\\blog_qa\\hello.dll.log", LevelFilter::Info);

        match find_window_by_pid(GetCurrentProcessId()) {
            Ok(handle) => {
                let result = SetWindowLongPtrW(handle, GWLP_WNDPROC, wnd_proc as isize);
                PREV_WNDPROC = transmute::<_, WNDPROC>(result);
                return true;
            },
            Err(e) => {
                error!("Error attaching hello.dll: {:?}", e);
            }
        };
    };
    false
}

fn detach() -> bool {
    unsafe {
        match find_window_by_pid(GetCurrentProcessId()) {
            Ok(handle) => {
                SetWindowLongPtrW(handle, GWLP_WNDPROC, transmute::<WNDPROC, _>(PREV_WNDPROC));
                return true;
            },
            Err(e) => {
                error!("Error detaching hello.dll: {:?}", e);
            }
        };
    };
    false
}

fn is_maximized(hwnd: HWND) -> bool {
    let style = unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) as u32 };
    (style & !WS_MAXIMIZE.0) != style

}

struct SWP {
    cmd: SET_WINDOW_POS_FLAGS,
    name: &'static str
}

fn check_style(flags: SET_WINDOW_POS_FLAGS) {

    let styles = vec![
        SWP { cmd: SWP_HIDEWINDOW     , name: "SWP_HIDEWINDOW" },
        SWP { cmd: SWP_NOACTIVATE     , name: "SWP_NOACTIVATE" },
        SWP { cmd: SWP_NOCOPYBITS     , name: "SWP_NOCOPYBITS" },
        SWP { cmd: SWP_NOMOVE         , name: "SWP_NOMOVE" },
        SWP { cmd: SWP_NOOWNERZORDER  , name: "SWP_NOOWNERZORDER" },
        SWP { cmd: SWP_NOREDRAW       , name: "SWP_NOREDRAW" },
        SWP { cmd: SWP_NOREPOSITION   , name: "SWP_NOREPOSITION" },
        SWP { cmd: SWP_NOSENDCHANGING , name: "SWP_NOSENDCHANG" },
        SWP { cmd: SWP_NOSIZE         , name: "SWP_NOSIZE" },
        SWP { cmd: SWP_NOZORDER       , name: "SWP_NOZORDER" },
        SWP { cmd: SWP_SHOWWINDOW     , name: "SWP_SHOWWINDOW" },
    ];

    let mut found = Vec::new();

    for i in styles {
        if (flags & !i.cmd ) != flags {
            found.push(i.name);
        }
    }
    info!("- flags: {:?}", found);
}

// WS_BORDER
// WS_CAPTION
// WS_CHILD
// WS_CHILDWINDOW
// WS_CLIPCHILDREN
// WS_CLIPSIBLINGS
// WS_DISABLED
// WS_DLGFRAME
// WS_GROUP
// WS_HSCROLL
// WS_ICONIC
// WS_MAXIMIZE
// WS_MAXIMIZEBOX
// WS_MINIMIZE
// WS_MINIMIZEBOX
// WS_OVERLAPPED
// WS_OVERLAPPEDWIN
// WS_POPUP
// WS_POPUPWINDOW
// WS_SIZEBOX
// WS_SYSMENU
// WS_TABSTOP
// WS_THICKFRAME
// WS_TILED
// WS_TILEDWINDOW
// WS_VISIBLE
// WS_VSCROLL

static mut once:bool = false;

extern "system" fn wnd_proc(
	window: HWND,
	message: u32,
	wparam: WPARAM,
	lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match message {
            // WM_WINDOWPOSCHANGING events are generated when a window is being sized
            // or moved.  If the window is maximized, we override the default behavior
            // and manually define the window's size and position.
            WM_WINDOWPOSCHANGING => {
                let data = lparam.0 as *mut WINDOWPOS;
                let data = data.as_mut().unwrap();

                // The system sets the WS_MAXIMIZE style prior to posting a
                // WM_WINDOWPOSCHANGING message, which is convenient for us...
                if is_maximized(window) && !once {
                    info!("[WM_WINDOWPOSCHANGING] hwnd: {:#x}, hwndInsertAfter: {:#x}, x: {}, y: {}, cx: {}, cy: {}",
                        data.hwnd.0,
                        data.hwndInsertAfter.0,
                        data.x,
                        data.y,
                        data.cx,
                        data.cy,
                    );
                    check_style(data.flags);
                    // once = true;
                    data.flags |= SWP_NOSIZE | SWP_NOMOVE;
                    return LRESULT(0);
                }
                // else if !is_maximized(window) && once {
                //     once = false;
                // }
            }
            WM_STYLECHANGING => {
                let data = lparam.0 as *mut STYLESTRUCT;
                let data = data.as_mut().unwrap();

                info!("WM_STYLECHANGING: styleOld {:?}", dbg!(data.styleOld));
                info!("WM_STYLECHANGING: styleNew {:?}", dbg!(data.styleNew));
                return LRESULT(0);
            }
            WM_NCCALCSIZE => {
                if IsZoomed(window).as_bool() {
                    let nc_params = lparam.0 as *mut NCCALCSIZE_PARAMS;
                    let nc_params = nc_params.as_mut().unwrap();
                    let rg_rcs = nc_params.rgrc;

                    // I have no clue why this works, but it prevents Chrome and Brave's
                    // titlebars from quirking, while also allowing GitKraken's titlebar
                    // to not disappear...
                    let r2 = rg_rcs[1];
                    let r3 = rg_rcs[2];
                    if r2.top == r3.top {
                        return LRESULT((WVR_ALIGNTOP | WVR_HREDRAW | WVR_VREDRAW) as _);
                    }
                }
            }
            WM_NCDESTROY => {
                info!("WM_NCDESTROY");
                let result = transmute::<WNDPROC, _>(PREV_WNDPROC);
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
