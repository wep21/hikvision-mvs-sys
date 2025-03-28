//! # hikvision_mvs_sys
//!
//! `hikvision_mvs_sys` is a Rust binding for the Hikvision MVS SDK.
//!
//! ## Features
//! - Initialize the MVS SDK
//! - Enumerate and list connected cameras
//! - Open a camera and configure settings
//!
//! ## Example
//! ```no_run
//! use hikvision_mvs_sys::*;
//! use std::ptr;
//!
//! unsafe {
//!     // Initialize SDK
//!     if MV_CC_Initialize() != MV_OK as i32 {
//!         eprintln!("Failed to initialize SDK.");
//!         return;
//!     }
//!
//!     let mut device_list: MV_CC_DEVICE_INFO_LIST = MV_CC_DEVICE_INFO_LIST {
//!         nDeviceNum: 0,
//!         pDeviceInfo: [ptr::null_mut(); 256],
//!     };
//!
//!     // Enumerate devices
//!     if MV_CC_EnumDevices(MV_GIGE_DEVICE | MV_USB_DEVICE, &mut device_list) != MV_OK as i32 {
//!         eprintln!("Failed to enumerate devices.");
//!         return;
//!     }
//! }
//! ```

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
