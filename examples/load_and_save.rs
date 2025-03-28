use hikvision_mvs_sys::*;
use std::ffi::CStr;
use std::io;
use std::io::Write;
use std::ptr;

fn main() {
    unsafe {
        // Initialize SDK
        if MV_CC_Initialize() != MV_OK as i32 {
            eprintln!("Failed to initialize SDK.");
            return;
        }

        let mut device_list: MV_CC_DEVICE_INFO_LIST = MV_CC_DEVICE_INFO_LIST {
            nDeviceNum: 0,
            pDeviceInfo: [ptr::null_mut(); 256],
        };

        // Enumerate devices
        if MV_CC_EnumDevices(MV_GIGE_DEVICE | MV_USB_DEVICE, &mut device_list) != MV_OK as i32 {
            eprintln!("Failed to enumerate devices.");
            return;
        }

        if device_list.nDeviceNum == 0 {
            println!("No devices found.");
            return;
        }

        println!("Found {} devices:", device_list.nDeviceNum);

        for i in 0..device_list.nDeviceNum as usize {
            let device_info = &*device_list.pDeviceInfo[i];

            if device_info.nTLayerType == MV_GIGE_DEVICE {
                println!(
                    "Device Model Name: {:?}",
                    CStr::from_ptr(
                        device_info.SpecialInfo.stGigEInfo.chModelName.as_ptr() as *const i8
                    )
                );
                println!(
                    "User Defined Name: {:?}",
                    CStr::from_ptr(
                        device_info
                            .SpecialInfo
                            .stGigEInfo
                            .chUserDefinedName
                            .as_ptr() as *const i8
                    )
                );
            } else if device_info.nTLayerType == MV_USB_DEVICE {
                println!(
                    "Device Model Name: {:?}",
                    CStr::from_ptr(
                        device_info.SpecialInfo.stUsb3VInfo.chModelName.as_ptr() as *const i8
                    )
                );
                println!(
                    "User Defined Name: {:?}",
                    CStr::from_ptr(
                        device_info
                            .SpecialInfo
                            .stUsb3VInfo
                            .chUserDefinedName
                            .as_ptr() as *const i8
                    )
                );
            }
        }

        // Get user input
        let index = loop {
            print!(
                "Enter the camera index to open (0-{}): ",
                device_list.nDeviceNum - 1
            );
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            match input.trim().parse::<usize>() {
                Ok(num) if num < device_list.nDeviceNum as usize => break num,
                _ => println!("Invalid input. Please enter a valid index."),
            }
        };

        let device_info = &*device_list.pDeviceInfo[index];

        let mut handle: *mut std::ffi::c_void = ptr::null_mut();

        // Create handle
        if MV_CC_CreateHandle(&mut handle, device_info) != MV_OK as i32 {
            eprintln!("Failed to create device handle.");
            return;
        }

        // Open device (requires additional arguments: access mode and mode)
        if MV_CC_OpenDevice(handle, MV_ACCESS_Exclusive, 0) != MV_OK as i32 {
            eprintln!("Failed to open device.");
            return;
        }

        // Save device features to file
        let feature_file = std::ffi::CString::new("FeatureFile.ini").unwrap();
        if MV_CC_FeatureSave(handle, feature_file.as_ptr()) != MV_OK as i32 {
            eprintln!("Failed to save device features.");
        }

        // Load device features from file
        if MV_CC_FeatureLoad(handle, feature_file.as_ptr()) != MV_OK as i32 {
            eprintln!("Failed to load device features.");
        }

        // Close device
        MV_CC_CloseDevice(handle);
        MV_CC_DestroyHandle(handle);

        println!("Device operations completed successfully.");
    }
}
