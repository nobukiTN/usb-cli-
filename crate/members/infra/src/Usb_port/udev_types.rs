use udev::{EventType,};
use domain::usb_domain::usb_model::{UsbId,UsbInfo};
use domain::usb_domain::usb_port::{UsbEvent,UsbEventKind};
#[derive(Debug)]
struct RawUdevProps {
    vendor_id: Option<String>,
    product_id: Option<String>,
    serial: Option<String>,
}

fn map_event(e: udev::Event) -> UsbEvent {
    use udev::EventType;
    let kind = match e.event_type() {
        EventType::Add => UsbEventKind::Added,
        EventType::Remove => UsbEventKind::Removed,
        _ => UsbEventKind::Changed,
    };
    let devnode = e.device().devnode().map(|p| p.to_path_buf());
    UsbEvent { kind, devnode, info: None }
}

fn map_props_to_usb_info(p: RawUdevProps) -> UsbInfo {
    UsbInfo::new(p.serial, p.vendor_id.unwrap_or_default(), p.product_id.unwrap_or_default())
}
