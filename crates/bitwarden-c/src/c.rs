use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    str,
};

use bitwarden_json::client::Client;
use tokio::task::JoinHandle;

use crate::{box_ptr, ffi_ref};

#[repr(C)]
pub struct CClient {
    /// Associates the tokio runtime to the `Client`, ensuring the runtime has the same lifecycle
    /// as the `Client`.
    runtime: tokio::runtime::Runtime,
    client: Client,
}

#[no_mangle]
pub extern "C" fn run_command(c_str_ptr: *const c_char, client_ptr: *const CClient) -> *mut c_char {
    let client = unsafe { ffi_ref!(client_ptr) };
    let input_str = str::from_utf8(unsafe { CStr::from_ptr(c_str_ptr) }.to_bytes())
        .expect("Input should be a valid string");

    let result = client
        .runtime
        .block_on(client.client.run_command(input_str));

    match std::ffi::CString::new(result) {
        Ok(cstr) => cstr.into_raw(),
        Err(_) => panic!("failed to return command result: null encountered"),
    }
}

type OnCompletedCallback = unsafe extern "C" fn(result: *mut c_char) -> ();

#[no_mangle]
pub extern "C" fn run_command_async(
    c_str_ptr: *const c_char,
    client_ptr: *const CClient,
    on_completed_callback: OnCompletedCallback,
    is_cancellable: bool,
) -> *mut JoinHandle<()> {
    let client = unsafe { ffi_ref!(client_ptr) };
    let input_str = str::from_utf8(unsafe { CStr::from_ptr(c_str_ptr) }.to_bytes())
        .expect("Input should be a valid string")
        // Languages may assume that the string is collectable as soon as this method exits
        // but it's not since the request will be run in the background
        // so we need to make our own copy.
        .to_owned();

    let join_handle = client.runtime.spawn(async move {
        let result = client.client.run_command(input_str.as_str()).await;
        let str_result = match std::ffi::CString::new(result) {
            Ok(cstr) => cstr.into_raw(),
            Err(_) => panic!("failed to return comment result: null encountered"),
        };

        // run completed function
        unsafe {
            on_completed_callback(str_result);
            let _ = CString::from_raw(str_result);
        }
    });

    // We only want to box the join handle the caller has said that they may want to cancel,
    // essentially promising to us that they will take care of the returned pointer.
    if is_cancellable {
        box_ptr!(join_handle)
    } else {
        std::ptr::null_mut()
    }
}

// Init client, potential leak! You need to call free_mem after this!
#[no_mangle]
pub extern "C" fn init(c_str_ptr: *const c_char) -> *mut CClient {
    // This will only fail if another logger was already initialized, so we can ignore the result
    let _ = env_logger::try_init();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build tokio runtime");

    let client = if c_str_ptr.is_null() {
        Client::new(None)
    } else {
        let input_string = str::from_utf8(unsafe { CStr::from_ptr(c_str_ptr) }.to_bytes())
            .expect("Input should be a valid string")
            .to_owned();
        Client::new(Some(input_string))
    };

    box_ptr!(CClient { runtime, client })
}

// Free mem
#[no_mangle]
pub extern "C" fn free_mem(client_ptr: *mut CClient) {
    std::mem::drop(unsafe { Box::from_raw(client_ptr) });
}

#[no_mangle]
pub extern "C" fn abort_and_free_handle(join_handle_ptr: *mut tokio::task::JoinHandle<()>) -> () {
    let join_handle = unsafe { Box::from_raw(join_handle_ptr) };
    join_handle.abort();
    std::mem::drop(join_handle);
}

#[no_mangle]
pub extern "C" fn free_handle(join_handle_ptr: *mut tokio::task::JoinHandle<()>) -> () {
    std::mem::drop(unsafe { Box::from_raw(join_handle_ptr) });
}
