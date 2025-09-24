use std::sync::mpsc;
use std::thread;

use udev::{EventType, MonitorBuilder};
use domain::usb_domain::usb_port::{UsbEventSource, UsbEvent, UsbEventKind};
use domain::usb_domain::usb_error::UsbError;

pub struct UdevUsbEventSource {
    subsystem: String,
}

impl UdevUsbEventSource {
    pub fn new(subsystem: impl Into<String>) -> Result<Self, UsbError> {
        Ok(Self { subsystem: subsystem.into() })
    }
}

impl UsbEventSource for UdevUsbEventSource {
    fn watch(
        &self,
    ) -> Result<Box<dyn Iterator<Item = Result<UsbEvent, UsbError>> + Send>, UsbError> {

        let (tx, rx) = mpsc::sync_channel::<Result<UsbEvent, UsbError>>(256);
        let subsystem = self.subsystem.clone();


        thread::Builder::new()
            .name("udev-monitor".into())
            .spawn(move || {
                let socket = match MonitorBuilder::new()
                    .and_then(|b| b.match_subsystem(&subsystem))
                    .and_then(|b| b.listen())
                {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = tx.send(Err(UsbError::AnyError(e.to_string())));
                        return;
                    }
                };

                for evt in socket.iter() {
                    let kind = match evt.event_type() {
                        EventType::Add => UsbEventKind::Added,
                        EventType::Remove => UsbEventKind::Removed,
                        _ => UsbEventKind::Changed,
                    };
                    let devnode = evt.device().devnode().map(|p| p.to_path_buf());
                    let info = None;

                    if tx.send(Ok(UsbEvent { kind, devnode, info })).is_err() {
                        break; 
                    }
                }
            })
            .map_err(|e| UsbError::AnyError(format!("spawn failed: {e}")))?;


        struct RxIter<T> {
            rx: mpsc::Receiver<T>,
        }
        impl<T> Iterator for RxIter<T> {
            type Item = T;
            fn next(&mut self) -> Option<Self::Item> {
                self.rx.recv().ok()
            }
        }

        Ok(Box::new(RxIter { rx }))
    }
}
