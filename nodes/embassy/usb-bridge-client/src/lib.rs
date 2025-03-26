#![no_std]

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_usb::control::{InResponse, OutResponse, Recipient, RequestType};
use embassy_usb::types::InterfaceNumber;
use embassy_usb::UsbDevice;
use protocol::{affector, usb};

const MAX_RECV_ITEM_SIZE: usize = 1 + affector::Affector::ENCODED_SIZE;

pub(crate) type RecvItem = heapless::Vec<u8, MAX_RECV_ITEM_SIZE>;
type SendQueue =
    Mutex<NoopRawMutex, heapless::Deque<u8, { usb::SEND_BUFFER_SIZE }>>;

pub struct UsbControlHandler<'a> {
    pub if_num: Option<InterfaceNumber>,
    affector_list: &'a [u8],
    send_queue: &'a SendQueue,
    ready_to_send: &'a Signal<NoopRawMutex, ()>,
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

        // Accept affector orders only.
        if req.request == usb::AFFECTOR_ORDER
            && data.len() <= MAX_RECV_ITEM_SIZE
        {
            let data =
                heapless::Vec::from_slice(data).expect("checked length above");
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

        if buf.len() != usb::SEND_BUFFER_SIZE {
            defmt::warn!(
                "Send buffer is incorrect size ({}), should be: {}. \
                recompile either usb-bridge or this",
                buf.len(),
                usb::SEND_BUFFER_SIZE
            );
            return Some(InResponse::Rejected);
        }

        if req.request == usb::GET_AFFECTOR_LIST {
            self.send_affector_list(buf);
            Some(InResponse::Accepted(buf))
        } else if req.request == usb::GET_QUEUED_MESSAGES {
            self.send_queued(buf)?;
            Some(InResponse::Accepted(buf))
        } else {
            Some(InResponse::Rejected)
        }
    }
}

impl UsbControlHandler<'_> {
    fn send_affector_list(&mut self, buf: &mut [u8]) {
        buf[0] = protocol::Msg::<5>::AFFECTOR_LIST;
        buf[1..1 + self.affector_list.len()]
            .copy_from_slice(self.affector_list);
        buf[1 + self.affector_list.len()..].fill(0);
    }

    fn send_queued(&mut self, buf: &mut [u8]) -> Option<()> {
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
        self.ready_to_send.signal(());
        Some(())
    }
}

pub struct UsbHandle<'a> {
    send_queue: &'a SendQueue,
    ready_to_send: &'a Signal<NoopRawMutex, ()>,
    receive_queue:
        &'a Channel<NoopRawMutex, heapless::Vec<u8, MAX_RECV_ITEM_SIZE>, 2>,
}

impl<'a> UsbHandle<'a> {
    pub fn split(&self) -> (UsbSender<'a>, UsbReceiver<'a>) {
        (
            UsbSender {
                send_queue: self.send_queue,
                ready_to_send: self.ready_to_send,
            },
            UsbReceiver {
                receive_queue: self.receive_queue,
            },
        )
    }
}

pub struct UsbSender<'a> {
    send_queue: &'a SendQueue,
    ready_to_send: &'a Signal<NoopRawMutex, ()>,
}

pub struct NoSpaceInQueue;

impl UsbSender<'_> {
    pub async fn send(
        &self,
        to_send: &[u8],
        is_low_prio: bool,
    ) -> Result<(), NoSpaceInQueue> {
        const RESERVED_FOR_HIGH_PRIO: usize = 32;

        let mut queue = self.send_queue.lock().await;
        let free = queue.capacity() - queue.len();
        if free < 1 + to_send.len() {
            return Err(NoSpaceInQueue);
        }

        let left_over = free - 1 - to_send.len();
        if left_over < RESERVED_FOR_HIGH_PRIO && is_low_prio {
            return Err(NoSpaceInQueue);
        }

        defmt::unwrap!(
            queue.push_back(defmt::unwrap!(
                to_send.len().try_into(),
                "send only supports buffers up to and including u8::MAX long"
            ),),
            "just checked that queue has capacity"
        );

        for byte in to_send {
            defmt::unwrap!(queue.push_back(*byte))
        }
        Ok(())
    }

    pub async fn wait_till_queue_free(&self) {
        let wait_for_ready = self.ready_to_send.wait();
        if !self.send_queue.lock().await.is_full() {
            return;
        }

        wait_for_ready.await
    }
}

#[allow(dead_code)]
pub struct UsbReceiver<'a> {
    receive_queue:
        &'a Channel<NoopRawMutex, heapless::Vec<u8, MAX_RECV_ITEM_SIZE>, 2>,
}
#[allow(dead_code)]
impl UsbReceiver<'_> {
    pub async fn recv(&self) -> RecvItem {
        self.receive_queue.receive().await
    }
}

/// Given a node name and a *random* serial returns a usb config
#[macro_export]
macro_rules! config {
    ($node_name:literal, $serial:literal) => {
        {
            let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
            config.manufacturer = Some("Vid");
            config.product = Some(concat!("sensor node ", $node_name));
            // Random, host driver finds device using this
            config.serial_number = Some($serial);
            // Required for windows compatibility.
            // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
            config.device_class = 0xEF;
            config.device_sub_class = 0x02;
            config.device_protocol = 0x01;
            config.composite_with_iads = true;
            config
        }
    };
}

pub struct StackA {
    send_queue: SendQueue,
    ready_to_send: Signal<NoopRawMutex, ()>,
    receive_queue:
        Channel<NoopRawMutex, heapless::Vec<u8, MAX_RECV_ITEM_SIZE>, 2>,
}

impl StackA {
    pub fn new() -> Self {
        Self {
            send_queue: Mutex::new(heapless::Deque::new()),
            receive_queue: Channel::new(),
            ready_to_send: Signal::new(),
        }
    }
}

pub struct StackB<'a> {
    config_descriptor: [u8; 256],
    bos_descriptor: [u8; 256],
    msos_descriptor: [u8; 256],
    control_buf: [u8; usb::SEND_BUFFER_SIZE],
    handler: UsbControlHandler<'a>,
}

impl<'a> StackB<'a> {
    pub fn new(stack_a: &'a StackA, affector_list: &'a [u8]) -> Self {
        Self {
            config_descriptor: [0u8; 256],
            bos_descriptor: [0u8; 256],
            msos_descriptor: [0u8; 256],
            control_buf: [0u8; usb::SEND_BUFFER_SIZE],

            handler: UsbControlHandler {
                if_num: None,
                send_queue: &stack_a.send_queue,
                receive_queue: &stack_a.receive_queue,
                ready_to_send: &stack_a.ready_to_send,
                affector_list,
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

pub fn new<'a, D: embassy_usb::driver::Driver<'a>>(
    stack_a: &'a StackA,
    stack_b: &'a mut StackB<'a>,
    config: embassy_usb::Config<'static>,
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
        config,
        config_descriptor,
        bos_descriptor,
        msos_descriptor,
        control_buf,
    );

    // Add the Microsoft OS Descriptor (MSOS/MOD) descriptor.
    // We tell Windows that this entire device is compatible with the "WINUSB" feature.
    // This causes it to use the built-in WInUSB driver automatically, which in turn
    // can be used by libusb/rusb software without needing a custom driver or INF file.
    // In principle you might want to call msos_feature() just on a specific function,
    // if your device also has other functions that still use standard class drivers.
    builder.msos_descriptor(windows_version::WIN8_1, 0);
    builder
        .msos_feature(msos::CompatibleIdFeatureDescriptor::new("WINUSB", ""));

    // Randomly generated UUID because Windows requires you provide one to use WinUSB.
    // In principle WinUSB-using software could find this device (or a specific interface
    // on it) by its GUID instead of using the VID/PID, but in practice that seems unhelpful.
    const DEVICE_INTERFACE_GUIDS: &[&str] =
        &["{DAC2087C-63FA-458D-A55D-827C0762DEC7}"];

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
            ready_to_send: &stack_a.ready_to_send,
        },
    )
}
