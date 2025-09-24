use std::fmt::format;
use std::os::unix::fs::MetadataExt; 
use std::task::ready;
use std::path::{Path,PathBuf};
use domain::usb_domain::{usb_error::*,usb_model::*,usb_port::*};
use time::OffsetDateTime;
use uuid::{Uuid,Timestamp,NoContext};
use udev::{Device,Enumerator,MonitorBuilder,MonitorSocket,Event,EventType,};

pub struct GeneratorId;
impl CreateId for GeneratorId{
    fn generate_v7_id(&self) -> Uuid {
        let now = OffsetDateTime::now_utc();
        let ts = Timestamp::from_unix(
            NoContext,now.unix_timestamp() as u64,now.nanosecond(),);
        let id = Uuid::new_v7(ts);
        id
    }
    fn generate_v4_id(&self) ->Uuid {
        let id = Uuid::new_v4();
        id
    }
}

pub struct SysClock;

impl Clock for SysClock{
    fn now_local(&self) -> Result<OffsetDateTime,UsbError> {
        let now = OffsetDateTime::now_local()
        .map_err(|e| UsbError::AnyError(format!("cannot get local_time: {e}")))?;
        Ok(now)
    }
    fn now_utc(&self) -> Result<OffsetDateTime,UsbError> {
        let now = OffsetDateTime::now_utc();
        Ok(now)
    }
}

pub struct UdevUsbProbe{
    udev : udev::Udev,
}

impl UdevUsbProbe{
    pub fn new()-> Result<Self,UsbError>{
        Ok(Self{udev:udev::Udev::new().map_err(|e| UsbError::AnyError(e.to_string()))?})
    }
       fn device_from_devnode(&self, dev: &Path) -> Result<Device, UsbError> {
        let md = std::fs::metadata(dev)
            .map_err(|e| UsbError::AnyError(format!("metadata({}): {e}", dev.display())))?;
        let rdev = md.rdev(); 

    Device::from_devnum(udev::DeviceType::Block, rdev)
            .map_err(|e| UsbError::AnyError(format!("from_devnum({}): {e}", dev.display())))
    }


fn read_usb_info(&self, dev: &Device) -> UsbInfo {
    let mut vid = prop_string(dev, "ID_VENDOR_ID")
        .or_else(|| attr_string(dev, "idVendor"))
        .unwrap_or_default();

    let mut pid = prop_string(dev, "ID_MODEL_ID")
        .or_else(|| attr_string(dev, "idProduct"))
        .unwrap_or_default();

    let mut serial = prop_string(dev, "ID_SERIAL_SHORT")
        .or_else(|| attr_string(dev, "serial"));

    let mut cur = Some(dev.clone());
    while serial.is_none() || vid.is_empty() || pid.is_empty() {
        cur = cur.and_then(|d| d.parent());
        let Some(ref p) = cur else { break; };

 
        if vid.is_empty() {
            if let Some(v) = prop_string(p, "ID_VENDOR_ID").or_else(|| attr_string(p, "idVendor")) {
                vid = v;
            }
        }
        if pid.is_empty() {
            if let Some(v) = prop_string(p, "ID_MODEL_ID").or_else(|| attr_string(p, "idProduct")) {
                pid = v;
            }
        }
        if serial.is_none() {
            serial = prop_string(p, "ID_SERIAL_SHORT").or_else(|| attr_string(p, "serial"));
        }
    }

    UsbInfo { vendor: vid, pid, serial }
}

}


impl UsbProbe for UdevUsbProbe{
    fn probe_by_devnode(&self, dev: &Path) -> Result<UsbInfo, UsbError> {
        let device = self.device_from_devnode(dev)?;
        Ok(self.read_usb_info(&device))
    }
    fn list_all(&self) -> Result<Vec<(PathBuf, UsbInfo)>, UsbError> {
        let mut en = Enumerator::new()
        .map_err(|e| UsbError::AnyError(e.to_string()))?;
     
        en.match_subsystem("block")
            .map_err(|e| UsbError::AnyError(e.to_string()))?;
   
        en.match_property("ID_BUS", "usb")
            .map_err(|e| UsbError::AnyError(e.to_string()))?;
      

        let mut out = Vec::new();
        for dev in en
            .scan_devices()
            .map_err(|e| UsbError::AnyError(e.to_string()))?
        {
            if let Some(devnode) = dev.devnode() {
             
                if let Some(dt) = dev.property_value("DEVTYPE").and_then(|s| s.to_str()) {
                    if dt == "partition" {
                        continue;
                    }
                }
                let info = self.read_usb_info(&dev);
                out.push((devnode.to_path_buf(), info));
            }
        }
        Ok(out)
    }
}
fn prop_string(d: &Device, key: &str) -> Option<String> {
    d.property_value(key)
        .and_then(|s| s.to_str().map(|x| x.to_owned()))
}

fn attr_string(d: &Device, key: &str) -> Option<String> {
    d.attribute_value(key)
        .and_then(|s| s.to_str().map(|x| x.to_owned()))
}