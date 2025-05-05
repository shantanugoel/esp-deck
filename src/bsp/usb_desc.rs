use std::os::raw::c_char;

use esp_idf_svc::sys::tusb_desc_device_t;

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct KeyboardReport {
    pub modifier: u8,
    pub reserved: u8,
    pub keys: [u8; 6],
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct MouseReport {
    pub buttons: u8, // Bit 0: Left, Bit 1: Right, Bit 2: Middle
    pub x: i8,       // Movement in X direction (-127 to 127)
    pub y: i8,       // Movement in Y direction (-127 to 127)
    pub wheel: i8,   // Wheel movement (-127 to 127)
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct ConsumerReport {
    pub usage: u16, // Consumer Usage ID
}

// Report IDs
pub const REPORT_ID_KEYBOARD: u8 = 1;
pub const REPORT_ID_MOUSE: u8 = 2;
pub const REPORT_ID_CONSUMER: u8 = 3;

// Update this if you change TUSB_DESC_HID_REPORT
const REPORT_DESCRIPTOR_LEN: u16 = 137;

// HID Report Descriptor
pub const TUSB_DESC_HID_REPORT: [u8; REPORT_DESCRIPTOR_LEN as usize] = [
    // --- Keyboard TLC ---
    0x05, 0x01, // Usage Page (Generic Desktop)
    0x09, 0x06, // Usage (Keyboard)
    0xA1, 0x01, // Collection (Application)
    0x85, 0x01, //   Report ID (1) <-- KEYBOARD ID = 1
    // Keyboard Input Report Items (Modifiers, Reserved, 6 Keys)
    0x05, 0x07, //   Usage Page (Key Codes)
    0x19, 0xE0, //   Usage Minimum (Left Ctrl)
    0x29, 0xE7, //   Usage Maximum (Right GUI)
    0x15, 0x00, //   Logical Minimum (0)
    0x25, 0x01, //   Logical Maximum (1)
    0x75, 0x01, //   Report Size (1) bit
    0x95, 0x08, //   Report Count (8) bits
    0x81, 0x02, //   Input (Data, Variable, Absolute) ; Modifier Byte
    0x95, 0x01, //   Report Count (1)
    0x75, 0x08, //   Report Size (8) bits
    0x81, 0x01, //   Input (Constant) ; Reserved Byte
    0x95, 0x06, //   Report Count (6)
    0x75, 0x08, //   Report Size (8) bits
    0x15, 0x00, //   Logical Minimum (0)
    0x26, 0xFF, 0x00, //   Logical Maximum (255) ; Adjust if needed
    0x05, 0x07, //   Usage Page (Key Codes)
    0x19, 0x00, //   Usage Minimum (Reserved)
    0x29, 0xFF, //   Usage Maximum (possibly Keyboard Lang.) ; Adjust range as needed
    0x81, 0x00, //   Input (Data, Array) ; Keycode array (6 bytes)
    // Optional: Keyboard Output Report for LEDs (Num lock, Caps lock etc)
    // 0x95, 0x05,       //   Report Count (5) ; Num, Caps, Scroll, Compose, Kana
    // 0x75, 0x01,       //   Report Size (1)
    // 0x05, 0x08,       //   Usage Page (LEDs)
    // 0x19, 0x01,       //   Usage Minimum (Num Lock)
    // 0x29, 0x05,       //   Usage Maximum (Kana)
    // 0x91, 0x02,       //   Output (Data, Variable, Absolute) ; LED Report
    // 0x95, 0x01,       //   Report Count (1)
    // 0x75, 0x03,       //   Report Size (3)
    // 0x91, 0x03,       //   Output (Constant) ; LED Report Padding
    0xC0, // End Collection (Keyboard)
    // --- Mouse TLC ---
    0x05, 0x01, // Usage Page (Generic Desktop)
    0x09, 0x02, // Usage (Mouse)
    0xA1, 0x01, // Collection (Application)
    0x85, 0x02, //   Report ID (2) <-- MOUSE ID = 2
    0x09, 0x01, //   Usage (Pointer)
    0xA1, 0x00, //   Collection (Physical)
    // Mouse Buttons (e.g., 3 buttons)
    0x05, 0x09, //     Usage Page (Button)
    0x19, 0x01, //     Usage Minimum (Button 1)
    0x29, 0x03, //     Usage Maximum (Button 3)
    0x15, 0x00, //     Logical Minimum (0)
    0x25, 0x01, //     Logical Maximum (1)
    0x75, 0x01, //     Report Size (1) bit
    0x95, 0x03, //     Report Count (3) buttons
    0x81, 0x02, //     Input (Data, Variable, Absolute) ; Button states
    0x95, 0x01, //     Report Count (1)
    0x75, 0x05, //     Report Size (5) bits
    0x81, 0x03, //     Input (Constant) ; Padding to fill byte
    // Mouse X, Y Movement (Relative)
    0x05, 0x01, //     Usage Page (Generic Desktop)
    0x09, 0x30, //     Usage (X)
    0x09, 0x31, //     Usage (Y)
    0x15, 0x81, //     Logical Minimum (-127)
    0x25, 0x7F, //     Logical Maximum (127)
    0x75, 0x08, //     Report Size (8) bits
    0x95, 0x02, //     Report Count (2) ; X, Y
    0x81, 0x06, //     Input (Data, Variable, Relative) ; X, Y movement
    // Optional: Mouse Wheel
    0x09, 0x38, //     Usage (Wheel)
    0x15, 0x81, //     Logical Minimum (-127)
    0x25, 0x7F, //     Logical Maximum (127)
    0x75, 0x08, //     Report Size (8)
    0x95, 0x01, //     Report Count (1) ; Wheel
    0x81, 0x06, //     Input (Data, Variable, Relative) ; Wheel movement
    0xC0, //   End Collection (Physical)
    0xC0, // End Collection (Mouse)
    // --- Consumer Control TLC ---
    0x05, 0x0C, // Usage Page (Consumer Devices)
    0x09, 0x01, // Usage (Consumer Control)
    0xA1, 0x01, // Collection (Application)
    0x85, 0x03, //   Report ID (3) <-- CONSUMER ID = 3
    // Define the Consumer Control Input Report (typically sends one 16-bit Usage ID at a time)
    0x19, 0x00, //   Usage Minimum (0) - Start range low
    0x2A, 0x3C,
    0x02, //   Usage Maximum (0x23C - AC Desktop Sleep) - Needs to cover all IDs you might send
    0x15, 0x00, //   Logical Minimum (0)
    0x26, 0x3C, 0x02, //   Logical Maximum (0x23C or higher)
    0x75, 0x10, //   Report Size (16) bits - Consumer Usage IDs are often 16-bit
    0x95, 0x01, //   Report Count (1) - Send one Usage ID per report
    0x81, 0x00, //   Input (Data, Array) ; The Consumer Usage ID
    0xC0, // End Collection (Consumer Control)
];

// !!! MUST BE >= LARGEST REPORT DATA SIZE + 1 (for Report ID prefix) !!!
// Keyboard=8 bytes data -> min 9. Mouse=4 -> min 5. Consumer=2 -> min 3.
// But we also need to account for bulk endpoints, so let's use 64 which is common for bulk endpoints
const MAX_PACKET_SIZE: u16 = 64;
// Polling interval for the HID Interrupt IN endpoint (in milliseconds)
const ENDPOINT_POLL_MS: u8 = 10;
// Configuration String Index (0 for none)
const CONFIG_STRING_INDEX: u8 = 0;
// Interface String Index (0 for none)
const INTERFACE_STRING_INDEX_HID: u8 = 4;
const INTERFACE_STRING_INDEX_VENDOR: u8 = 5;

// Power attributes
const USB_CONFIG_ATTR: u8 = 0xA0; // Bus powered + Remote Wakeup (0x80 = Bus powered only)
const USB_MAX_POWER_MA: u8 = 100; // Max power in mA

// --- Calculated Total Configuration Descriptor Size ---
// Config (9) + Interface (9) + HID (9) + Endpoint (7) + Vendor (9) + Vendor Endpoint  Out (7)  + Vendor Endpoint In (7) = 57 bytes
const CONFIG_DESC_TOTAL_LEN: u16 = 57;

// WebUSB and MS OS 2.0 constants
pub const ITF_NUM_VENDOR: u8 = 1;
pub const VENDOR_REQUEST_WEBUSB: u8 = 1;
pub const VENDOR_REQUEST_MICROSOFT: u8 = 2;

pub const TURB_REQUEST_TYPE_STANDARD: u8 = 0;
pub const TURB_REQUEST_TYPE_CLASS: u8 = 1;
pub const TURB_REQUEST_TYPE_VENDOR: u8 = 2;
pub const TURB_REQUEST_TYPE_INVALID: u8 = 3;

// Endpoint assignments. 0 is control, HID uses 1 IN
const EP_VENDOR_OUT: u8 = 0x02;
const EP_VENDOR_IN: u8 = 0x82;

#[rustfmt::skip]
pub const TUSB_DESC_CONFIGURATION: [u8; CONFIG_DESC_TOTAL_LEN as usize] = [
    // --- Configuration Descriptor (9 bytes) ---
    0x09,                                       // bLength: Size of this descriptor (9 bytes)
    usb_constants::descriptor_type::CONFIGURATION, // bDescriptorType: CONFIGURATION (0x02)
    (CONFIG_DESC_TOTAL_LEN & 0xFF) as u8,       // wTotalLength (Low Byte): Total length (Config + Interface + HID + Endpoint)
    (CONFIG_DESC_TOTAL_LEN >> 8) as u8,         // wTotalLength (High Byte)
    0x02,                                       // bNumInterfaces: 2 interfaces (HID + Vendor)
    0x01,                                       // bConfigurationValue: Configuration value 1
    CONFIG_STRING_INDEX,                        // iConfiguration: Index of string descriptor (0 = None)
    USB_CONFIG_ATTR,                            // bmAttributes: (e.g., 0xA0 = Bus powered + Remote Wakeup)
    (USB_MAX_POWER_MA / 2) as u8,               // bMaxPower: Max power in 2mA units (e.g., 100mA -> 50)

    // HID Interface: Interface 0
    // --- Interface Descriptor (9 bytes) ---
    0x09,                                       // bLength: Size of this descriptor (9 bytes)
    usb_constants::descriptor_type::INTERFACE,   // bDescriptorType: INTERFACE (0x04)
    0x00,                                       // bInterfaceNumber: Interface Number 0
    0x00,                                       // bAlternateSetting: Alternate Setting 0
    0x01,                                       // bNumEndpoints: 1 endpoint for this interface
    usb_constants::class_code::HID,             // bInterfaceClass: HID (0x03)
    0x00,                                       // bInterfaceSubClass: No Subclass (0x01 for Boot Interface)
    0x00,                                       // bInterfaceProtocol: None (0x01 for Keyboard, 0x02 for Mouse Boot Protocol)
    INTERFACE_STRING_INDEX_HID,                 // iInterface: Index of string descriptor (0 = None)

    // --- HID Class Descriptor (9 bytes) ---
    0x09,                                       // bLength: Size of this descriptor (9 bytes)
    usb_constants::descriptor_type::HID,        // bDescriptorType: HID (0x21)
    0x11, 0x01,                                 // bcdHID: HID Class Specification release number (1.11) LSB, MSB
    0x00,                                       // bCountryCode: Hardware target country (0 = None)
    0x01,                                       // bNumDescriptors: Number of HID class descriptors to follow (1 = Report Descriptor)
    usb_constants::descriptor_type::REPORT,     // bDescriptorType: Type of descriptor = REPORT (0x22)
    (REPORT_DESCRIPTOR_LEN & 0xFF) as u8,       // wDescriptorLength (Low Byte): Total length of the HID Report Descriptor
    (REPORT_DESCRIPTOR_LEN >> 8) as u8,         // wDescriptorLength (High Byte)

    // --- Endpoint Descriptor (7 bytes) ---
    0x07,                                       // bLength: Size of this descriptor (7 bytes)
    usb_constants::descriptor_type::ENDPOINT,    // bDescriptorType: ENDPOINT (0x05)
    0x81,                                       // bEndpointAddress: Endpoint 1, IN direction (MSB=1 for IN)
    usb_constants::endpoint_attribute::INTERRUPT,// bmAttributes: Interrupt transfer type (0x03)
    (MAX_PACKET_SIZE & 0xFF) as u8,             // wMaxPacketSize (Low Byte): Max packet size (e.g., 16 bytes)
    (MAX_PACKET_SIZE >> 8) as u8,               // wMaxPacketSize (High Byte)
    ENDPOINT_POLL_MS,                           // bInterval: Polling interval in ms for Interrupt Endpoint

    // Vendor Interface: Interface 1
    // --- Interface Descriptor (9 bytes) ---
    0x09,                                       // bLength: Size of this descriptor (9 bytes)
    usb_constants::descriptor_type::INTERFACE,   // bDescriptorType: INTERFACE (0x04)
    ITF_NUM_VENDOR,                              // bInterfaceNumber: Interface Number 1
    0x00,                                       // bAlternateSetting: Alternate Setting 0
    0x02,                                       // bNumEndpoints: 2 endpoints for this interface (Out/In)
    usb_constants::class_code::VENDOR_SPEC,          // bInterfaceClass: Vendor (0xEF)
    0x00,                                       // bInterfaceSubClass: No Subclass (0x02 for Vendor Class)
    0x00,                                       // bInterfaceProtocol: None (0x01 for Vendor Class)
    INTERFACE_STRING_INDEX_VENDOR,              // iInterface: Index of string descriptor (0 = None)

    // --- Endpoint Descriptor Out (7 bytes) ---
    0x07,                                       // bLength: Size of this descriptor (7 bytes)
    usb_constants::descriptor_type::ENDPOINT,   // bDescriptorType: ENDPOINT (0x05)
    EP_VENDOR_OUT,                              // bEndpointAddress: Endpoint 2, OUT direction (MSB=0 for OUT)
    usb_constants::endpoint_attribute::BULK,    // bmAttributes: Bulk transfer type (0x02)
    (MAX_PACKET_SIZE & 0xFF) as u8,             // wMaxPacketSize (Low Byte): Max packet size (e.g., 16 bytes)
    (MAX_PACKET_SIZE >> 8) as u8,               // wMaxPacketSize (High Byte)
    0x00,                                       // bInterval: Ignored for Bulk endpoints

    // --- Endpoint Descriptor In (7 bytes) ---
    0x07,                                       // bLength: Size of this descriptor (7 bytes)
    usb_constants::descriptor_type::ENDPOINT,   // bDescriptorType: ENDPOINT (0x05)
    EP_VENDOR_IN,                              // bEndpointAddress: Endpoint 2, IN direction (MSB=1 for IN)
    usb_constants::endpoint_attribute::BULK,    // bmAttributes: Bulk transfer type (0x02)
    (MAX_PACKET_SIZE & 0xFF) as u8,             // wMaxPacketSize (Low Byte): Max packet size (e.g., 16 bytes)
    (MAX_PACKET_SIZE >> 8) as u8,               // wMaxPacketSize (High Byte)
    0x00,                                       // bInterval: Ignored for Bulk endpoints
    
];

// Device Descriptor
const USB_VID: u16 = 0x5AA6;
const USB_PID: u16 = 0x60E1;
const USB_DEVICE_VERSION: u16 = 0x0001;

pub const TUSB_DESC_DEVICE: tusb_desc_device_t = tusb_desc_device_t {
    // --- Device Descriptor (18 bytes) ---
    bLength: 18, // bLength: Size of this descriptor (18 bytes)
    bDescriptorType: usb_constants::descriptor_type::DEVICE, // bDescriptorType: DEVICE (0x01)
    bcdUSB: 0x0201, // bcdUSB: USB Specification Release Number (2.1) to suoport webusb
    // --- Class/SubClass/Protocol: Typically set for composite device / IAD ---
    // Option 1: Use Interface Association Descriptor values (Common for composite)
    bDeviceClass: 0xEF,    // bDeviceClass: Miscellaneous (0xEF)
    bDeviceSubClass: 0x02, // bDeviceSubClass: Common Class (0x02)
    bDeviceProtocol: 0x01, // bDeviceProtocol: Interface Association Descriptor (0x01)
    // Option 2: Defined at Interface level (Also common)
    // 0x00,                                          // bDeviceClass: Defined in Interface Descriptor (0x00)
    // 0x00,                                          // bDeviceSubClass: Defined in Interface Descriptor (0x00)
    // 0x00,                                          // bDeviceProtocol: Defined in Interface Descriptor (0x00)
    bMaxPacketSize0: 64, // bMaxPacketSize0: Max packet size for Endpoint 0 (64 bytes)
    idVendor: USB_VID,   // idVendor (Low Byte, High Byte)
    idProduct: USB_PID,  // idProduct (Low Byte, High Byte)
    bcdDevice: USB_DEVICE_VERSION, // bcdDevice (Device Release Number)
    // --- String Descriptor Indices (Typically 1, 2, 3 when strings are enabled) ---
    // These indices map to the strings you set in sdkconfig
    iManufacturer: 1, // iManufacturer: Index of Manufacturer String Descriptor
    iProduct: 2,      // iProduct: Index of Product String Descriptor
    iSerialNumber: 3, // iSerialNumber: Index of Serial Number String Descriptor
    bNumConfigurations: 0x01, // bNumConfigurations: Number of possible configurations (Usually 1)
};

// // Helper macro to extract the least significant byte (LSB) from a 16-bit value
// macro_rules! lsb {
//     ($val:expr) => {
//         ($val & 0xFF) as u8
//     };
// }

// // Helper macro to extract the most significant byte (MSB) from a 16-bit value
// macro_rules! msb {
//     ($val:expr) => {
//         (($val >> 8) & 0xFF) as u8
//     };
// }

// // Device Descriptor
// const USB_VID: u16 = 0x5AA4;
// const USB_PID: u16 = 0x60E1;
// const USB_DEVICE_VERSION: u16 = 0x0100;

// // Device descriptor (18 bytes)
// pub const TUSB_DESC_DEVICE: [u8; 18] = [
//     18,                                     // bLength: Size of this descriptor
//     usb_constants::descriptor_type::DEVICE, // bDescriptorType: DEVICE
//     lsb!(0x0200),
//     msb!(0x0200), // bcdUSB: USB 2.00
//     // Class/SubClass/Protocol: IAD used
//     0xEF, // bDeviceClass: Miscellaneous
//     0x02, // bDeviceSubClass: Common Class
//     0x01, // bDeviceProtocol: Interface Association Descriptor
//     64,   // bMaxPacketSize0: Max packet size for Endpoint 0
//     lsb!(USB_VID),
//     msb!(USB_VID), // idVendor
//     lsb!(USB_PID),
//     msb!(USB_PID), // idProduct
//     lsb!(USB_DEVICE_VERSION),
//     msb!(USB_DEVICE_VERSION), // bcdDevice
//     1,                        // iManufacturer: Index of Manufacturer String Descriptor
//     2,                        // iProduct: Index of Product String Descriptor
//     3,                        // iSerialNumber: Index of Serial Number String Descriptor
//     1,                        // bNumConfigurations: Number of possible configurations
// ];

// --- Helper structs for USB Constants ---
pub mod usb_constants {
    pub mod descriptor_type {
        pub const DEVICE: u8 = 0x01;
        pub const CONFIGURATION: u8 = 0x02;
        pub const STRING: u8 = 0x03;
        pub const INTERFACE: u8 = 0x04;
        pub const ENDPOINT: u8 = 0x05;
        pub const HID: u8 = 0x21;
        pub const REPORT: u8 = 0x22;
        pub const PHYSICAL: u8 = 0x23;
        pub const BOS: u8 = 0x0F;
        pub const DEVICE_CAPABILITY: u8 = 0x10;
    }
    pub mod class_code {
        pub const HID: u8 = 0x03;
        pub const VENDOR_SPEC: u8 = 0xFF;
    }
    pub mod endpoint_attribute {
        pub const CONTROL: u8 = 0x00;
        pub const ISOCHRONOUS: u8 = 0x01;
        pub const BULK: u8 = 0x02;
        pub const INTERRUPT: u8 = 0x03;
    }
    pub mod capability_type {
        pub const PLATFORM: u8 = 0x05;
    }
}

// Check struct sizes at compile time (not necessary, but good practice)
const _: () = assert!(std::mem::size_of::<KeyboardReport>() == 8);
const _: () = assert!(std::mem::size_of::<MouseReport>() == 4);
const _: () = assert!(std::mem::size_of::<ConsumerReport>() == 2);

// --- Add BOS Descriptor ---
// Correct BOS Calculation: BOS Header (5) + WebUSB Cap (24) + MS OS Cap (28) = 57
const BOS_TOTAL_LEN: u16 = 5 + 24 + 28;

#[rustfmt::skip]
pub const TUSB_DESC_BOS: [u8; BOS_TOTAL_LEN as usize] = [
    // BOS Descriptor
    0x05, usb_constants::descriptor_type::BOS, (BOS_TOTAL_LEN & 0xFF) as u8, (BOS_TOTAL_LEN >> 8) as u8, 0x02, // bNumDeviceCaps = 2

    // WebUSB Platform Capability Descriptor (Length = 24)
    0x18, usb_constants::descriptor_type::DEVICE_CAPABILITY, usb_constants::capability_type::PLATFORM, 0x00,
    0x38, 0xB6, 0x08, 0x34, 0xA9, 0x09, 0xA0, 0x47, 0x8B, 0xFD, 0xA0, 0x76, 0x88, 0x15, 0xB6, 0x65, // GUID
    0x00, 0x01, // bcdVersion 1.00
    VENDOR_REQUEST_WEBUSB, // bVendorCode
    1, // iLandingPage (Using index 1 for URL descriptor, needs to be defined)

    // Microsoft OS 2.0 Platform Capability Descriptor (Length = 28)
    0x1C, usb_constants::descriptor_type::DEVICE_CAPABILITY, usb_constants::capability_type::PLATFORM, 0x00,
    0xDF, 0x60, 0xDD, 0xD8, 0x89, 0x45, 0xC7, 0x4C, 0x9C, 0xD2, 0x65, 0x9D, 0x9E, 0x64, 0x8A, 0x9F, // GUID
    0x00, 0x00, 0x03, 0x06, // dwWindowsVersion >= Win 8.1
    // wDescriptorSetTotalLength = 168
    (MS_OS_20_DESC_LEN & 0xFF) as u8, (MS_OS_20_DESC_LEN >> 8) as u8,
    VENDOR_REQUEST_MICROSOFT, // bVendorCode
    0x00 // bAltEnumCode
];

// === Microsoft OS 2.0 Descriptor Set ===
// This is fetched by Windows via vendor specific request
const MS_OS_20_DESC_LEN: u16 = 10 + 8 + 20 + 140; // CSet Header (10) + Function Subset (8) + Compatible ID (20) + Registry Property (140)

#[rustfmt::skip]
pub const TUSB_DESC_MS_OS_20: [u8; MS_OS_20_DESC_LEN as usize] = [
  // Set header: length, type, windows version, total length
  0x0A, 0x00, // wLength = 10
  0x00, 0x00, // MS OS 2.0 descriptor set header
  0x00, 0x00, 0x03, 0x06, // dwWindowsVersion >= Win 8.1
  (MS_OS_20_DESC_LEN & 0xFF) as u8, (MS_OS_20_DESC_LEN >> 8) as u8, // wTotalLength 

  // Configuration Subset header: length, type, config index, reserved, subset length
  0x08, 0x00, // wLength = 8
  0x01, 0x00, // MS OS 2.0 configuration subset header
  0x00, 0x00, // bConfigurationValue = 0
  // wSubsetLength = 150
  ((MS_OS_20_DESC_LEN - 0xA) & 0xFF) as u8, ((MS_OS_20_DESC_LEN - 0xA) >> 8) as u8,

  // Function Subset header: length, type, first interface, reserved, subset length
  0x08, 0x00, // wLength = 8
  0x02, 0x00, // MS OS 2.0 function subset header
  ITF_NUM_VENDOR, // bFirstInterface = Interface 1 (Vendor)
  0x00,       // reserved
  // wSubsetLength = Compatible ID (20) + Reg Property (130) = 150
  ((MS_OS_20_DESC_LEN - 0xA - 0x8) & 0xFF) as u8, ((MS_OS_20_DESC_LEN - 0xA - 0x8) >> 8) as u8,

  // Compatible ID Descriptor: length, type, compatible ID, sub compatible ID
  0x14, 0x00, // wLength = 20
  0x03, 0x00, // MS OS 2.0 compatible ID descriptor
  b'W', b'I', b'N', b'U', b'S', b'B', 0x00, 0x00, // CompatibleID "WINUSB"
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // SubCompatibleID

  // Registry Property Descriptor: length, type, property data type, property name length, property name, property data length, property data
  // GUID_DEVINTERFACE_USB_DEVICE = {A5DCBF10-6530-11D2-901F-00C04FB951ED}
  // Corrected wLength = 130 bytes (DataType(2) + NameLength(2) + Name(42) + DataLength(2) + Data(80))
  ((MS_OS_20_DESC_LEN - 0x0A - 0x08 - 0x08 - 0x14) & 0xFF) as u8, ((MS_OS_20_DESC_LEN - 0x0A - 0x08 - 0x08 - 0x14) >> 8) as u8, 
  0x04, 0x00,
  0x07, 0x00, // wPropertyDataType = 7 (REG_MULTI_SZ)
  0x2A, 0x00, // wPropertyNameLength = 42
  // Property Name: DeviceInterfaceGUIDs
  b'D', 0x00, b'e', 0x00, b'v', 0x00, b'i', 0x00, b'c', 0x00, b'e', 0x00, b'I', 0x00,
  b'n', 0x00, b't', 0x00, b'e', 0x00, b'r', 0x00, b'f', 0x00, b'a', 0x00, b'c', 0x00,
  b'e', 0x00, b'G', 0x00, b'U', 0x00, b'I', 0x00, b'D', 0x00, b's', 0x00, 0x00, 0x00, // Terminating NUL

  0x50, 0x00, // wPropertyDataLength = 80 bytes
  //bPropertyData: “{975F44D9-0D08-43FD-8B3E-127CA8AFFF9D}”.
  b'{', 0x00, b'9', 0x00, b'7', 0x00, b'5', 0x00, b'F', 0x00, b'4', 0x00, b'4', 0x00,
  b'D', 0x00, b'9', 0x00, b'-', 0x00, b'0', 0x00, b'D', 0x00, b'0', 0x00, b'8', 0x00,
  b'-', 0x00, b'4', 0x00, b'3', 0x00, b'F', 0x00, b'D', 0x00, b'-', 0x00, b'8', 0x00,
  b'B', 0x00, b'3', 0x00, b'E', 0x00, b'-', 0x00, b'1', 0x00, b'2', 0x00, b'7', 0x00,
  b'C', 0x00, b'A', 0x00, b'8', 0x00, b'A', 0x00, b'F', 0x00, b'F', 0x00, b'F', 0x00,
  b'9', 0x00, b'D', 0x00, b'}', 0x00, 0x00, 0x00, 0x00, 0x00 // Double NUL termination
];

// === URL Descriptor for WebUSB Landing Page ===
const LANDING_PAGE_URL: &[u8] = b"espdeckcfg.shantanugoel.com"; // The actual URL string bytes
const URL_DESC_LEN: usize = 3 + LANDING_PAGE_URL.len(); // 3 bytes header (len, type, scheme) + URL length

#[rustfmt::skip]
pub const TUSB_DESC_WEBUSB_URL: [u8; URL_DESC_LEN] = {
    let mut desc = [0u8; URL_DESC_LEN];
    desc[0] = URL_DESC_LEN as u8;                        // bLength
    desc[1] = usb_constants::descriptor_type::STRING;    // bDescriptorType = 0x03 (String)
    desc[2] = 1;                                         // bScheme = 1 (https)
    // Copy URL bytes into the descriptor starting from index 3
    let mut i = 3;
    while i < URL_DESC_LEN {
        desc[i] = LANDING_PAGE_URL[i - 3];
        i += 1;
    }
    desc
};

pub static LANGUAGE_STRING: &[u8] = b"0409\0";
pub static MANUFACTURER_STRING: &[u8] = b"Shaan Labs Inc.\0";
pub static PRODUCT_STRING: &[u8] = b"ESP DECK\0";
pub static SERIAL_STRING: &[u8] = b"42069\0";
pub static INTERFACE_HID_STRING: &[u8] = b"ESP DECK HID Interface\0";
pub static INTERFACE_VENDOR_STRING: &[u8] = b"ESP DECK WebUSB Interface\0";
pub const STRING_DESCRIPTOR_LEN: usize = 6;

pub static mut STRING_DESCRIPTOR: [*const c_char; STRING_DESCRIPTOR_LEN] = [
    LANGUAGE_STRING.as_ptr(),
    MANUFACTURER_STRING.as_ptr(),
    PRODUCT_STRING.as_ptr(),
    SERIAL_STRING.as_ptr(),
    INTERFACE_HID_STRING.as_ptr(),
    INTERFACE_VENDOR_STRING.as_ptr(),
];
