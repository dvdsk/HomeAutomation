use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_usb::control::{InResponse, OutResponse, Recipient, RequestType};
use embassy_usb::types::InterfaceNumber;
use embassy_usb::UsbDevice;
use protocol::affector;

/// USB full speed devices such as this one have a max data packet size of 1023
const SEND_BUFFER_SIZE: usize = 208;
const MAX_RECV_ITEM_SIZE: usize = 1 + affector::Affector::ENCODED_SIZE;

pub(crate) type RecvItem = heapless::Vec<u8, MAX_RECV_ITEM_SIZE>;

pub struct UsbControlHandler<'a> {
    pub if_num: Option<InterfaceNumber>,
    send_queue: &'a Mutex<NoopRawMutex, heapless::Deque<u8, SEND_BUFFER_SIZE>>,
    receive_queue: &'a Channel<NoopRawMutex, RecvItem, 2>,
}

impl embassy_usb::Handler for UsbControlHandler<'_> {
    fn control_out(
        &mut self,
        req: embassy_usb::control::Request,
        data: &[u8],
    ) -> Option<embassy_usb::control::OutResponse> {
        // Log the request before filtering to help with debugging.
        defmt::info!("Got control_out, request={}, buf={:a}", req, data);

        // Only handle Vendor request types to an Interface.
        if req.request_type != RequestType::Vendor {
            return None;
        }

        if req.recipient != Recipient::Interface {
            return None;
        }

        // Ignore requests to other interfaces.
        if req.index != defmt::unwrap!(self.if_num).0 as u16 {
            return None;
        }

        // Accept request 100, value 200, reject others.
        if req.request == 100 && req.value == 200 && data.len() <= MAX_RECV_ITEM_SIZE {
            let data = heapless::Vec::from_slice(data).expect("checked length above");
            if self.receive_queue.try_send(data).is_err() {
                Some(OutResponse::Rejected)
            } else {
                Some(OutResponse::Accepted)
            }
        } else {
            Some(OutResponse::Rejected)
        }
    }

    fn control_in<'a>(
        &'a mut self,
        req: embassy_usb::control::Request,
        buf: &'a mut [u8],
    ) -> Option<InResponse<'a>> {
        // Only handle Vendor request types to an Interface.
        if req.request_type != RequestType::Vendor {
            return None;
        }

        if req.recipient != Recipient::Interface {
            return None;
        }

        // Ignore requests to other interfaces.
        if req.index != defmt::unwrap!(self.if_num).0 as u16 {
            return None;
        }

        if buf.len() != SEND_BUFFER_SIZE {
            defmt::warn!(
                "Send buffer is incorrect size ({}), should be: {}. \
                Please adjuct host driver (usb-bridge) os the usb stack here",
                buf.len(),
                SEND_BUFFER_SIZE
            );
            return Some(InResponse::Rejected);
        }

        let Ok(mut send_queue) = self.send_queue.try_lock() else {
            return None;
        };

        for byte in buf.iter_mut() {
            if let Some(to_send) = send_queue.pop_front() {
                *byte = to_send;
            } else {
                *byte = 0; // end of encoded data
            }
        }

        Some(InResponse::Accepted(buf))
    }
}

pub struct UsbHandle<'a> {
    send_queue: &'a Mutex<NoopRawMutex, heapless::Deque<u8, SEND_BUFFER_SIZE>>,
    receive_queue: &'a Channel<NoopRawMutex, heapless::Vec<u8, MAX_RECV_ITEM_SIZE>, 2>,
}

impl<'a> UsbHandle<'a> {
    pub(crate) fn split(&self) -> (UsbSender<'a>, UsbReceiver<'a>) {
        (
            UsbSender {
                send_queue: self.send_queue,
            },
            UsbReceiver {
                receive_queue: self.receive_queue,
            },
        )
    }
}

pub struct UsbSender<'a> {
    send_queue: &'a Mutex<NoopRawMutex, heapless::Deque<u8, SEND_BUFFER_SIZE>>,
}

impl UsbSender<'_> {
    pub(crate) async fn send(&self, to_send: &[u8]) {
        let mut queue = self.send_queue.lock().await;
        let free = queue.capacity() - queue.len();
        if free < 1 + to_send.len() {
            defmt::trace!("dropping package because queue is full");
            return; // drop the package
        }

        defmt::unwrap!(queue.push_back(
            to_send
                .len()
                .try_into()
                .expect("send only supports buffers up to and including u8::MAX long"),
        ));

        for byte in to_send {
            defmt::unwrap!(queue.push_back(*byte))
        }
    }
}

pub struct UsbReceiver<'a> {
    receive_queue: &'a Channel<NoopRawMutex, heapless::Vec<u8, MAX_RECV_ITEM_SIZE>, 2>,
}
impl UsbReceiver<'_> {
    pub(crate) async fn recv(&self) -> RecvItem {
        self.receive_queue.receive().await
    }
}

pub fn config() -> embassy_usb::Config<'static> {
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Vid");
    config.product = Some(concat!("sensor node ", env!("CARGO_PKG_NAME")));
    config.serial_number = Some("2139874"); // random, host driver finds device using this

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    config
}

pub struct StackA {
    send_queue: Mutex<NoopRawMutex, heapless::Deque<u8, SEND_BUFFER_SIZE>>,
    receive_queue: Channel<NoopRawMutex, heapless::Vec<u8, MAX_RECV_ITEM_SIZE>, 2>,
}

impl StackA {
    pub(crate) fn new() -> Self {
        Self {
            send_queue: Mutex::new(heapless::Deque::new()),
            receive_queue: Channel::new(),
        }
    }
}

pub struct StackB<'a> {
    config_descriptor: [u8; 256],
    bos_descriptor: [u8; 256],
    msos_descriptor: [u8; 256],
    control_buf: [u8; SEND_BUFFER_SIZE],
    handler: UsbControlHandler<'a>,
}

impl<'a> StackB<'a> {
    pub(crate) fn new(stack_a: &'a StackA) -> Self {
        Self {
            config_descriptor: [0u8; 256],
            bos_descriptor: [0u8; 256],
            msos_descriptor: [0u8; 256],
            control_buf: [0u8; SEND_BUFFER_SIZE],

            handler: UsbControlHandler {
                if_num: None,
                send_queue: &stack_a.send_queue,
                receive_queue: &stack_a.receive_queue,
            },
        }
    }
}

pub struct Usb<'a, D: embassy_usb::driver::Driver<'a>>(UsbDevice<'a, D>);

impl<'a, D: embassy_usb::driver::Driver<'a>> Usb<'a, D> {
    pub async fn run(&mut self) {
        self.0.run().await
    }
}

pub(crate) fn new<'a, D: embassy_usb::driver::Driver<'a>>(
    stack_a: &'a StackA,
    stack_b: &'a mut StackB<'a>,
    driver: D,
) -> (Usb<'a, D>, UsbHandle<'a>) {
    use embassy_usb::msos::{self, windows_version};

    let StackB {
        config_descriptor,
        bos_descriptor,
        msos_descriptor,
        control_buf,
        handler,
    } = stack_b;

    let mut builder = embassy_usb::Builder::new(
        driver,
        config(),
        config_descriptor,
        bos_descriptor,
        msos_descriptor,
        control_buf,
    );

    // Add the Microsoft OS Descriptor (MSOS/MOD) descriptor.
    // We tell Windows that this entire device is compatible with the "WINUSB" feature,
    // which causes it to use the built-in WinUSB driver automatically, which in turn
    // can be used by libusb/rusb software without needing a custom driver or INF file.
    // In principle you might want to call msos_feature() just on a specific function,
    // if your device also has other functions that still use standard class drivers.
    builder.msos_descriptor(windows_version::WIN8_1, 0);
    builder.msos_feature(msos::CompatibleIdFeatureDescriptor::new("WINUSB", ""));

    // Randomly generated UUID because Windows requires you provide one to use WinUSB.
    // In principle WinUSB-using software could find this device (or a specific interface
    // on it) by its GUID instead of using the VID/PID, but in practice that seems unhelpful.
    const DEVICE_INTERFACE_GUIDS: &[&str] = &["{DAC2087C-63FA-458D-A55D-827C0762DEC7}"];

    builder.msos_feature(msos::RegistryPropertyFeatureDescriptor::new(
        "DeviceInterfaceGUIDs",
        msos::PropertyData::RegMultiSz(DEVICE_INTERFACE_GUIDS),
    ));

    let mut function = builder.function(0xFF, 0, 0);
    let mut interface = function.interface();
    let _alternate = interface.alt_setting(0xFF, 0, 0, None);
    handler.if_num = Some(interface.interface_number());
    drop(function);
    builder.handler(handler);

    (
        Usb(builder.build()),
        UsbHandle {
            send_queue: &stack_a.send_queue,
            receive_queue: &stack_a.receive_queue,
        },
    )
}
