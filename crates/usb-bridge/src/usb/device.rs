use color_eyre::eyre::{eyre, Context};
use color_eyre::Section;
use nusb::transfer;

pub(crate) fn get(request: u8) -> transfer::ControlIn {
    transfer::ControlIn {
        control_type: transfer::ControlType::Vendor,
        recipient: transfer::Recipient::Interface,
        request,
        value: 0,
        index: 0,
        length: protocol::usb::SEND_BUFFER_SIZE.try_into().expect("fits"),
    }
}

pub(crate) fn send(request: u8, data: &[u8]) -> transfer::ControlOut {
    transfer::ControlOut {
        control_type: transfer::ControlType::Vendor,
        recipient: transfer::Recipient::Interface,
        request,
        value: 0,
        index: 0,
        data,
    }
}

pub(crate) fn list_usb_devices(
    serial_number: &str,
) -> Result<Vec<nusb::DeviceInfo>, color_eyre::eyre::Error> {
    let list: Vec<_> = nusb::list_devices()
        .wrap_err("Could not list usb devices")?
        .filter(|d| {
            d.serial_number()
                .is_some_and(|d| d.eq_ignore_ascii_case(serial_number))
        })
        .collect();
    Ok(list)
}

pub(crate) fn get_usb_device(
    list: Vec<nusb::DeviceInfo>,
    serial_number: &str,
) -> Result<nusb::Device, color_eyre::eyre::Error> {
    match list.as_slice() {
        [dev] => dev,
        [] => {
            return Err(eyre!("No usb device found with the correct serial"))
                .with_note(|| {
                    format!("looking for device with serial: {serial_number}")
                })
                .suggestion(
                    "Is the device working (sometimes programming fails) \
                    & connected?",
                );
        }
        more => {
            return Err(eyre!(
                "Multiple usb devices have the same serial number"
            )
            .with_note(|| format!("they are: {more:?}")));
        }
    }
    .open()
    .wrap_err("Could not open the usb device")
    .suggestion("Try running as sudo")
    .with_suggestion(|| edit_udev_rules_instruction(serial_number))
}

fn edit_udev_rules_instruction(serial_number: &str) -> String {
    if let Some(group) = uzers::get_current_groupname() {
        let group = group.to_string_lossy();
        format!(
        "Add a .rules file in /etc/udev/rules.d with line: \n\
            ATTRS{{serial}}==\"{serial_number}\", MODE=\"660\", GROUP=\"{group}\", TAG+=\"uaccess\"\n\
            Then run: sudo udevadm control --reload && sudo udevadm trigger",
    )
    } else {
        "Look up this users primary group then add a .rules file 
            in /etc/udev/rules.d, replace <group> with the primary group: \n\
            ATTRS{{serial}}==\"{serial_number}\", MODE=\"660\", GROUP=\"<group>\", TAG+=\"uaccess\"\n\
            Then run: sudo udevadm control --reload && sudo udevadm trigger".to_string()
    }
}
