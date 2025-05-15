use hikvision_mvs_sys::*;
use std::ffi::{CStr, CString};
use std::io::{self, Write};
use std::ptr;

// Match the type of enPixelType (i64)
const PIXEL_TYPE_RGB8_PACKED: i64 = 0x02180014; // RGB8 packed format

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
                    "[{}] GigE Device - Model: {:?}, Name: {:?}",
                    i,
                    CStr::from_ptr(
                        device_info.SpecialInfo.stGigEInfo.chModelName.as_ptr() as *const i8
                    ),
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
                    "[{}] USB Device - Model: {:?}, Name: {:?}",
                    i,
                    CStr::from_ptr(
                        device_info.SpecialInfo.stUsb3VInfo.chModelName.as_ptr() as *const i8
                    ),
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

        // User input
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
                _ => println!("Invalid input. Try again."),
            }
        };

        let device_info = &*device_list.pDeviceInfo[index];
        let mut handle: *mut std::ffi::c_void = ptr::null_mut();

        // Create handle
        if MV_CC_CreateHandle(&mut handle, device_info) != MV_OK as i32 {
            eprintln!("Failed to create handle.");
            return;
        }

        // Open device
        if MV_CC_OpenDevice(handle, MV_ACCESS_Exclusive, 0) != MV_OK as i32 {
            eprintln!("Failed to open device.");
            MV_CC_DestroyHandle(handle);
            return;
        }

        // Load features
        let feature_file = CString::new("FeatureFile.ini").unwrap();
        if MV_CC_FeatureLoad(handle, feature_file.as_ptr()) != MV_OK as i32 {
            eprintln!("Failed to load feature file.");
        }

        // Start grabbing
        if MV_CC_StartGrabbing(handle) != MV_OK as i32 {
            eprintln!("Failed to start grabbing.");
            MV_CC_CloseDevice(handle);
            MV_CC_DestroyHandle(handle);
            return;
        }

        // Prepare buffer and frame info
        let mut frame_info: MV_FRAME_OUT_INFO_EX = std::mem::zeroed();
        let width = get_int_param(handle, "Width").unwrap_or(1920);
        let height = get_int_param(handle, "Height").unwrap_or(1080);
        let buffer_size = (width * height * 3) as usize; // assuming RGB8 packed
        let mut buffer = vec![0u8; buffer_size];

        let status = MV_CC_GetOneFrameTimeout(
            handle,
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            &mut frame_info,
            1000,
        );

        if status == MV_OK as i32 {
            println!(
                "Captured frame: {}x{}",
                frame_info.nWidth, frame_info.nHeight
            );
            save_frame_as_png(&buffer, &frame_info);
        } else {
            eprintln!("Failed to capture frame. Status: {}", status);
        }

        MV_CC_StopGrabbing(handle);
        MV_CC_CloseDevice(handle);
        MV_CC_DestroyHandle(handle);
        println!("Done.");
    }
}

fn save_frame_as_png(buffer: &[u8], frame_info: &MV_FRAME_OUT_INFO_EX) {
    let width = frame_info.nWidth as u32;
    let height = frame_info.nHeight as u32;
    let pixel_format = frame_info.enPixelType;

    match pixel_format {
        PIXEL_TYPE_RGB8_PACKED => {
            let img_size = (width * height * 3) as usize;
            if buffer.len() < img_size {
                eprintln!("Buffer too small for RGB image.");
                return;
            }

            use image::{ImageBuffer, Rgb};

            let img = ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, &buffer[..img_size])
                .expect("Failed to create ImageBuffer");

            if let Err(err) = img.save("frame.png") {
                eprintln!("Failed to save PNG: {}", err);
            } else {
                println!("Saved frame as frame.png");
            }
        }
        _ => {
            eprintln!("Unsupported pixel format: {:#X}", pixel_format);
        }
    }
}

unsafe fn get_int_param(handle: *mut std::ffi::c_void, key: &str) -> Option<u32> {
    let c_key = CString::new(key).unwrap();
    let mut value: MVCC_INTVALUE = std::mem::zeroed();

    // FIX: Correct argument order
    let status = MV_CC_GetIntValue(handle, c_key.as_ptr(), &mut value);

    if status == MV_OK as i32 {
        Some(value.nCurValue as u32)
    } else {
        eprintln!("Failed to get {}: error code {}", key, status);
        None
    }
}
