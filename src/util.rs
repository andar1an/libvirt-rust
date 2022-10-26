#[cfg(all(target_pointer_width = "64", not(windows)))]
pub fn c_ulong_to_u64(val: ::libc::c_ulong) -> u64 {
    val
}

#[cfg(not(all(target_pointer_width = "64", not(windows))))]
pub fn c_ulong_to_u64(val: ::libc::c_ulong) -> u64 {
    val as u64
}

#[allow(unused_macros)]
macro_rules! check_null {
    ($e:expr) => {{
        let ptr = $e;
        if ptr.is_null() {
            Err($crate::error::Error::last_error())
        } else {
            Ok(ptr)
        }
    }};
}

#[allow(unused_macros)]
macro_rules! check_neg {
    ($e:expr) => {{
        let ret = $e;
        if ret == -1 {
            Err($crate::error::Error::last_error())
        } else {
            Ok(ret)
        }
    }};
}

#[allow(unused_macros)]
macro_rules! check_zero {
    ($e:expr) => {{
        let ret = $e;
        if ret == 0 {
            Err($crate::error::Error::last_error())
        } else {
            Ok(ret)
        }
    }};
}

#[allow(unused_imports)]
pub(crate) use check_neg;
#[allow(unused_imports)]
pub(crate) use check_null;
#[allow(unused_imports)]
pub(crate) use check_zero;
