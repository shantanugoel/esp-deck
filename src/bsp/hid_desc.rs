use esp_idf_svc::sys::esptinyusb::tusb_desc_device_t;

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
// Max is 9, so 16 is a safe choice.
const MAX_PACKET_SIZE: u16 = 16;
// Polling interval for the HID Interrupt IN endpoint (in milliseconds)
const ENDPOINT_POLL_MS: u8 = 10;
// Configuration String Index (0 for none)
const CONFIG_STRING_INDEX: u8 = 0;
// Interface String Index (0 for none)
const INTERFACE_STRING_INDEX: u8 = 0;
// Power attributes
const USB_CONFIG_ATTR: u8 = 0xA0; // Bus powered + Remote Wakeup (0x80 = Bus powered only)
const USB_MAX_POWER_MA: u8 = 100; // Max power in mA

// --- Calculated Total Configuration Descriptor Size ---
// Config (9) + Interface (9) + HID (9) + Endpoint (7) = 34 bytes
const CONFIG_DESC_TOTAL_LEN: u16 = 34;

#[rustfmt::skip]
pub const TUSB_DESC_CONFIGURATION: [u8; CONFIG_DESC_TOTAL_LEN as usize] = [
    // --- Configuration Descriptor (9 bytes) ---
    0x09,                                       // bLength: Size of this descriptor (9 bytes)
    usb_constants::descriptor_type::CONFIGURATION, // bDescriptorType: CONFIGURATION (0x02)
    (CONFIG_DESC_TOTAL_LEN & 0xFF) as u8,       // wTotalLength (Low Byte): Total length (Config + Interface + HID + Endpoint)
    (CONFIG_DESC_TOTAL_LEN >> 8) as u8,         // wTotalLength (High Byte)
    0x01,                                       // bNumInterfaces: 1 interface
    0x01,                                       // bConfigurationValue: Configuration value 1
    CONFIG_STRING_INDEX,                        // iConfiguration: Index of string descriptor (0 = None)
    USB_CONFIG_ATTR,                            // bmAttributes: (e.g., 0xA0 = Bus powered + Remote Wakeup)
    (USB_MAX_POWER_MA / 2) as u8,               // bMaxPower: Max power in 2mA units (e.g., 100mA -> 50)

    // --- Interface Descriptor (9 bytes) ---
    0x09,                                       // bLength: Size of this descriptor (9 bytes)
    usb_constants::descriptor_type::INTERFACE,   // bDescriptorType: INTERFACE (0x04)
    0x00,                                       // bInterfaceNumber: Interface Number 0
    0x00,                                       // bAlternateSetting: Alternate Setting 0
    0x01,                                       // bNumEndpoints: 1 endpoint for this interface
    usb_constants::class_code::HID,             // bInterfaceClass: HID (0x03)
    0x00,                                       // bInterfaceSubClass: No Subclass (0x01 for Boot Interface)
    0x00,                                       // bInterfaceProtocol: None (0x01 for Keyboard, 0x02 for Mouse Boot Protocol)
    INTERFACE_STRING_INDEX,                     // iInterface: Index of string descriptor (0 = None)

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
];

// Device Descriptor
const USB_VID: u16 = 0x5AA4;
const USB_PID: u16 = 0x60E1;
const USB_DEVICE_VERSION: u16 = 0x0100;

pub const TUSB_DESC_DEVICE: tusb_desc_device_t = tusb_desc_device_t {
    // --- Device Descriptor (18 bytes) ---
    bLength: 18, // bLength: Size of this descriptor (18 bytes)
    bDescriptorType: usb_constants::descriptor_type::DEVICE, // bDescriptorType: DEVICE (0x01)
    bcdUSB: 0x0200, // bcdUSB: USB Specification Release Number (2.00) LSB, MSB
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
    }
    pub mod class_code {
        pub const HID: u8 = 0x03;
        // Add others if needed
    }
    pub mod endpoint_attribute {
        pub const CONTROL: u8 = 0x00;
        pub const ISOCHRONOUS: u8 = 0x01;
        pub const BULK: u8 = 0x02;
        pub const INTERRUPT: u8 = 0x03;
    }
}

// Check struct sizes at compile time (not necessary, but good practice)
const _: () = assert!(std::mem::size_of::<KeyboardReport>() == 8);
const _: () = assert!(std::mem::size_of::<MouseReport>() == 4);
const _: () = assert!(std::mem::size_of::<ConsumerReport>() == 2);
