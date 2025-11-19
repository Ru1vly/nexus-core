use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use rusqlite::Connection;

use crate::initialize_database;

/// Opaque pointer to a rusqlite Connection.
pub type DbConnection = Connection;

/// Initializes the database and returns a pointer to the connection.
///
/// # Safety
///
/// The caller is responsible for calling `ahenk_close_database` to free the connection.
/// The `db_path` must be a valid, null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ahenk_initialize_database(db_path: *const c_char) -> *mut DbConnection {
    if db_path.is_null() {
        return ptr::null_mut();
    }

    let c_str = CStr::from_ptr(db_path);
    let path = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    match initialize_database(path) {
        Ok(conn) => Box::into_raw(Box::new(conn)),
        Err(_) => ptr::null_mut(),
    }
}

/// Closes the database connection and frees the memory.
///
/// # Safety
///
/// The `conn_ptr` must be a valid pointer to a `DbConnection` that was created
/// by `ahenk_initialize_database`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ahenk_close_database(conn_ptr: *mut DbConnection) {
    if !conn_ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(conn_ptr);
        }
    }
}

/// Registers a new user.
///
/// # Safety
///
/// The `conn_ptr` must be a valid pointer to a `DbConnection`.
/// The `username`, `email`, and `password` must be valid, null-terminated C strings.
/// The caller is responsible for calling `ahenk_free_string` on the returned pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ahenk_register_user(
    conn_ptr: *mut DbConnection,
    username: *const c_char,
    email: *const c_char,
    password: *const c_char,
) -> *mut c_char {
    if conn_ptr.is_null() {
        return ptr::null_mut();
    }
    let conn = &*conn_ptr;

    let username = CStr::from_ptr(username).to_str().unwrap();
    let email = CStr::from_ptr(email).to_str().unwrap();
    let password = CStr::from_ptr(password).to_str().unwrap();

    match crate::logic::register_user(
        conn,
        username.to_string(),
        email.to_string(),
        password.to_string(),
    ) {
        Ok(user) => {
            let user_json = serde_json::to_string(&user).unwrap();
            CString::new(user_json).unwrap().into_raw()
        }
        Err(_) => ptr::null_mut(),
    }
}

/// Frees a string that was allocated by Rust.
///
/// # Safety
///
/// The `s_ptr` must be a valid pointer to a C string that was allocated by Rust.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ahenk_free_string(s_ptr: *mut c_char) {
    if !s_ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(s_ptr);
        }
    }
}

/// Logs in a user.
///
/// # Safety
///
/// The `conn_ptr` must be a valid pointer to a `DbConnection`.
/// The `username` and `password` must be valid, null-terminated C strings.
/// The caller is responsible for calling `ahenk_free_string` on the returned pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ahenk_login_user(
    conn_ptr: *mut DbConnection,
    username: *const c_char,
    password: *const c_char,
) -> *mut c_char {
    if conn_ptr.is_null() {
        return ptr::null_mut();
    }
    let conn = &*conn_ptr;

    let username = CStr::from_ptr(username).to_str().unwrap();
    let password = CStr::from_ptr(password).to_str().unwrap();

    match crate::logic::login_user(conn, username, password) {
        Ok(user) => {
            let user_json = serde_json::to_string(&user).unwrap();
            CString::new(user_json).unwrap().into_raw()
        }
        Err(_) => ptr::null_mut(),
    }
}
