use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::Channel;
use embassy_usb::control::{InResponse, OutResponse, Recipient, RequestType};
use embassy_usb::types::InterfaceNumber;
use embassy_usb::UsbDevice;
use protocol::{affector, ErrorReport};

const fn max(a: usize, b: usize) -> usize {
    [a, b][(a < b) as usize]
}

const MAX_SEND_ITEM_SIZE: usize = 1 + max(
    max(
        crate::comms::SensMsg::ENCODED_SIZE,
        ErrorReport::ENCODED_SIZE,
    ),
    affector::ListMessage::<5>::ENCODED_SIZE,
);

const MAX_RECV_ITEM_SIZE: usize = 1 + affector::Affector::ENCODED_SIZE;

pub(crate) type SendItem = heapless::Vec<u8, MAX_SEND_ITEM_SIZE>;
pub(crate) type RecvItem = heapless::Vec<u8, MAX_RECV_ITEM_SIZE>;

pub struct UsbControlHandler<'a> {
    pub if_num: Option<InterfaceNumber>,
    send_queue: &'a Channel<NoopRawMutex, SendItem, 2>,
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
        defmt::info!("Got control_in, request={}", req);

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

        let Ok(to_send) = self.send_queue.try_receive() else {
            return None;
        };

        if to_send.len() > buf.len() {
            return None;
        }

        buf[0..to_send.len()].copy_from_slice(&to_send);
        Some(InResponse::Accepted(buf))
    }
}

pub struct UsbHandle<'a> {
    send_queue: &'a Channel<NoopRawMutex, heapless::Vec<u8, MAX_SEND_ITEM_SIZE>, 2>,
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
    send_queue: &'a Channel<NoopRawMutex, heapless::Vec<u8, MAX_SEND_ITEM_SIZE>, 2>,
}

impl UsbSender<'_> {
    pub(crate) async fn send(&self, item: SendItem) {
        self.send_queue.send(item).await;
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
    config
}

pub struct StackA {
    send_queue: Channel<NoopRawMutex, heapless::Vec<u8, MAX_SEND_ITEM_SIZE>, 2>,
    receive_queue: Channel<NoopRawMutex, heapless::Vec<u8, MAX_RECV_ITEM_SIZE>, 2>,
}

impl StackA {
    pub(crate) fn new() -> Self {
        Self {
            send_queue: Channel::new(),
            receive_queue: Channel::new(),
        }
    }
}

pub struct StackB<'a> {
    config_descriptor: [u8; 256],
    bos_descriptor: [u8; 256],
    msos_descriptor: [u8; 256],
    control_buf: [u8; 64],
    handler: UsbControlHandler<'a>,
}

impl<'a> StackB<'a> {
    pub(crate) fn new(stack_a: &'a StackA) -> Self {
        Self {
            config_descriptor: [0u8; 256],
            bos_descriptor: [0u8; 256],
            msos_descriptor: [0u8; 256],
            control_buf: [0u8; 64],

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
