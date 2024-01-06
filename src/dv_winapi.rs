/* --> Summary of Contents

learning the winapi module

Summary of Contents <-- */
/* --> Imports */

use std::{thread, time::Duration};
use std::{ptr, ptr::null_mut};
use std::str::FromStr;
use winapi::{
    shared::{
		minwindef::{
			FALSE, UINT, 
			HINSTANCE, LPARAM, 
			LRESULT, WPARAM,
			DWORD, BOOL, TRUE,
			ATOM
		},
		windef::{ 
			HICON, HCURSOR, 
			HBRUSH, HWND, POINT
		},
	},
    um::{
        handleapi::CloseHandle,
        processthreadsapi::{
			CreateProcessW, PROCESS_INFORMATION, 
			STARTUPINFOW, TerminateProcess 
		},
        winbase::{ 
			CREATE_NEW_CONSOLE, INFINITE 
		},
        winnt::PROCESS_ALL_ACCESS,
		synchapi::WaitForSingleObject,
		winuser::{
			MessageBoxW, WNDCLASSEXW, WNDPROC, 
			CreateWindowExW, DefWindowProcW, GetMessageW, 
			LoadCursorW, PostQuitMessage, RegisterClassExW, 
			ShowWindow, TranslateMessage, DispatchMessageW, 
			MSG, CS_OWNDC, CS_HREDRAW, 
			CS_VREDRAW, WM_DESTROY, WS_OVERLAPPEDWINDOW, 
			WS_VISIBLE, IDC_ARROW, SW_SHOW, CW_USEDEFAULT,
			WS_CHILD, RedrawWindow, RDW_NOCHILDREN, RDW_UPDATENOW,
			RDW_INVALIDATE, RDW_ERASE, WM_MOVE, WM_SIZE, WM_COMMAND, 
			UpdateWindow, RDW_FRAME, COLOR_WINDOW, WS_BORDER, MoveWindow,
			WS_TABSTOP
		},
		libloaderapi::GetModuleHandleW,
		winnt::LPCWSTR,
		wingdi:: { 
			CreateSolidBrush, CreateBitmap 
		},
		commctrl:: { GetEffectiveClientRect, LVS_AUTOARRANGE, LVS_REPORT,
			INITCOMMONCONTROLSEX, InitCommonControlsEx, ICC_LISTVIEW_CLASSES,
			WC_LISTVIEW},
    },
	ctypes::c_int,
};

use widestring::U16CString;
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::io::Error;

/* <-- Imports */

/* --> Functions <-- */

pub fn entry_point() -> Result<(), ()> {
	let h_instance = unsafe { GetModuleHandleW(null_mut()) };
	let this_hwnd: HWND = build_new_window("WinApiAPP", h_instance);

	let icex = INITCOMMONCONTROLSEX { dwSize: 0 as DWORD, dwICC: ICC_LISTVIEW_CLASSES };
	let result = unsafe { InitCommonControlsEx(&icex) };
	
	let ok_btn_hwnd =	build_new_control("Ok", "Button", this_hwnd, h_instance, WS_TABSTOP | WS_VISIBLE | WS_CHILD, 10 as c_int, 10 as c_int, 80 as c_int, 15 as c_int);
	let mid_btn_hwnd =	build_new_control("Middle", "Button", this_hwnd, h_instance, WS_TABSTOP | WS_VISIBLE | WS_CHILD, 100 as c_int, 10 as c_int, 80 as c_int, 15 as c_int);
	let right_btn_hwnd =build_new_control("Right", "Button", this_hwnd, h_instance, WS_TABSTOP | WS_VISIBLE | WS_CHILD, 190 as c_int, 10 as c_int, 80 as c_int, 15 as c_int);
	/* //windows can't find "ListView" as a windows class name. try registering an atom instead
	let lv32_hwnd = 	build_new_control("", "ListView", this_hwnd, h_instance, WS_VISIBLE | WS_CHILD | LVS_REPORT, 
		10 as c_int, 	// x-pos
		35 as c_int, 	// y-pos
		400 as c_int, 	// width
		400 as c_int);	// height
*/
//	let result = move_window(lv32_hwnd as HWND, 0 as c_int, 0 as c_int, 400 as c_int, 400 as c_int, TRUE as BOOL);

	let mut msg = MSG {
		hwnd: null_mut(),
		message: 0,
		wParam: 0,
		lParam: 0,
		time: 0,
		pt: POINT { x: Default::default(), y: Default::default() },
	};
	
	let mut event_indicator: i32 = 1;
	
	while event_indicator > 0 { 
		event_indicator = main_window_event_loop(&mut msg);
	};
	println!("GUI thread is finished"); 
	Ok(())
}

fn terminate_proc(process_info: &mut PROCESS_INFORMATION, exit_code: u32) {
	unsafe {
		TerminateProcess(process_info.hProcess, exit_code);
		WaitForSingleObject(process_info.hProcess, INFINITE);
		CloseHandle(process_info.hThread);
		CloseHandle(process_info.hProcess);
	}
}

fn spawn_cmd() {
	let mut startup_info: STARTUPINFOW = unsafe { std::mem::zeroed() };
	let mut process_info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
	
	startup_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
	
	let command_line = U16CString::from_str("cmd.exe").unwrap();
	
	let success = unsafe {
		CreateProcessW(
		null_mut(),
		command_line.as_ptr() as *mut u16,
		null_mut(),
		null_mut(),
		FALSE,
		CREATE_NEW_CONSOLE,
		null_mut(),
		null_mut(),
		&mut startup_info,
		&mut process_info,
		)
	};
	
	if success == FALSE {
		println!("Failed to create process.");
		return;
	}
	
	println!("Process created successfully.");
	std::thread::sleep(Duration::from_secs(5));
	terminate_proc(&mut process_info, 0);
}

fn build_new_window(wintitle: &str, h_instance: HINSTANCE) -> HWND {
	let app_name = u16::from_str(wintitle).unwrap();
	let bg_brush = unsafe { CreateSolidBrush( 0x00FFFFFF ) };
	let raw_ptr_to_app_name = ptr::addr_of!(app_name);
	
//	let class_atom: WNDCLASSEXW = create_atom(
	let class_atom = create_atom(
		WS_BORDER | WS_VISIBLE, 
		Some(window_procedure),
		0,
		0,
		h_instance,
		null_mut(),
		unsafe { LoadCursorW(null_mut(), IDC_ARROW) },
		bg_brush,
		null_mut(),
		raw_ptr_to_app_name
		);

	let hwnd = unsafe {
		CreateWindowExW(
			0,                                   // dwExStyle
			class_atom as *const u16,			 // lpClassName
			raw_ptr_to_app_name,                   // lpWindowName
			WS_OVERLAPPEDWINDOW | WS_VISIBLE,    // dwStyle
			CW_USEDEFAULT,                       // X
			CW_USEDEFAULT,                       // Y
			CW_USEDEFAULT,                       // nWidth
			CW_USEDEFAULT,                       // nHeight
			null_mut(),                          // hWndParent
			null_mut(),                          // hMenu
			h_instance,                          // HINSTANCE
			null_mut(),                          // lpParam
		)
	};

	if hwnd.is_null() { return hwnd; }
	
	unsafe { ShowWindow(hwnd, SW_SHOW) };

	hwnd
}

fn create_atom(style_parameters: UINT, 
	windowprocedure: WNDPROC,
	class_extras: c_int,
	window_extras: c_int,
	handle_to_instance: HINSTANCE,
	handle_to_icon: HICON,
	handle_to_cursor: HCURSOR,
	handle_to_background: HBRUSH,
	menu_name: LPCWSTR,
	window_name: LPCWSTR	
	) -> ATOM {
	let handle_to_background = unsafe { CreateSolidBrush( 0x00FFFFFF ) };

	let raw_ptr_to_win_name = ptr::addr_of!(window_name);

	let w_class: WNDCLASSEXW = unsafe { WNDCLASSEXW {
		cbSize: 32 as UINT,				// UINT
		style: style_parameters,		// UINT
		lpfnWndProc: windowprocedure,	// WNDPROC
		cbClsExtra: class_extras,			// c_int,
		cbWndExtra: window_extras,			// c_int,
		hInstance: handle_to_instance,		// HINSTANCE,
		hIcon: null_mut(),					// HICON,
		hCursor: handle_to_cursor,	// HCURSOR,
		hbrBackground: handle_to_background,	// HBRUSH,
		lpszMenuName: null_mut(),				// LPCWSTR,
		lpszClassName: *raw_ptr_to_win_name,	// LPCWSTR,
		hIconSm: null_mut(),
	} };

	let class_atom = unsafe { RegisterClassExW(&w_class) };
	
	class_atom
}

fn build_new_control(
		wintitle: &str, 
		control: &str,
		parent: HWND, 
		h_instance: HINSTANCE, 
		dwStyle: DWORD,
		xpos: c_int, 
		ypos: c_int,
		width: c_int,
		height: c_int)
	-> HWND {
	let control_name = to_wstring(wintitle);
	let control_class = to_wstring(control);
	
	let control_hwnd = unsafe {
		CreateWindowExW(
			0,               // dwExStyle
			control_class.as_ptr(),				 // lpClassName
			control_name.as_ptr(),               // lpWindowName
			dwStyle,				   			 // dwStyle
			xpos,   	                    	 // X
			ypos,	    	                   	 // Y
			width,                     			 // nWidth
			height,                    			 // nHeight
			parent,                              // hWndParent
			null_mut(),                          // hMenu
			h_instance,                          // HINSTANCE
			null_mut(),                          // lpParam
		)
	};
	
	let os_error = Error::last_os_error();
	if control_hwnd.is_null() { println!("{os_error:?}") };

	if control_hwnd.is_null() { return control_hwnd; }

	unsafe { ShowWindow(control_hwnd, SW_SHOW) };

	control_hwnd
}

pub fn rust_MessageBox() {
	unsafe { MessageBoxW(null_mut(), null_mut(), null_mut(), 0) };
}

fn main_window_event_loop(msg: &mut MSG) -> i32 {
	let returnvalue = unsafe { GetMessageW(msg, null_mut(), 0, 0) };
	
	if returnvalue != 0 {
		unsafe {
			TranslateMessage(msg);
			DispatchMessageW(msg);
		}
	};
	returnvalue
}

unsafe extern "system" fn window_procedure(
	hwnd: HWND,
	msg: UINT,
	w_param: WPARAM,
	l_param: LPARAM,
	) -> LRESULT {
	match msg {
		WM_DESTROY => {
			PostQuitMessage(0);
			0
		},
		WM_MOVE => {
			//println!("Moved");
			0
		}, 
		WM_SIZE => {
			//println!("Resized");
			0
		}, 
		WM_COMMAND => {
			match l_param {
				_ => rust_MessageBox(),
			}
			0
		}, 
		_ => DefWindowProcW(hwnd, msg, w_param, l_param),
	}	
}

fn move_window(
	hWnd: HWND,
	x_pos: c_int,
	y_pos: c_int,
	nwidth: c_int,
	nheight: c_int,
	brepaint: BOOL) -> BOOL {
	unsafe { MoveWindow(hWnd, x_pos, y_pos,
				nwidth, nheight, brepaint) }
}

fn to_wstring(s: &str) -> Vec<u16> {
	OsStr::new(s)
		.encode_wide()
		.chain(once(0))
		.collect()
}