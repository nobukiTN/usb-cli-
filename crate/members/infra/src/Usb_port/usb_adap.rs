use std::fmt::format;
use std::fs::File;
use std::io::{BufRead,BufReader};
use nix::sys::stat::{major,minor};
use std::os::unix::fs::MetadataExt; 
use std::task::ready;
use std::path::{Path,PathBuf};
use domain::usb_domain::{usb_error::*,usb_model::*,usb_port::*};
use time::OffsetDateTime;
use uuid::{Uuid,Timestamp,NoContext};
use udev::{Device,Enumerator,MonitorBuilder,MonitorSocket,Event,EventType,};
use walkdir::{WalkDir,DirEntry};
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

pub struct ProcMountinfoResolver;

impl ProcMountinfoResolver {
    pub fn new() -> Self { Self }
}

impl MountPathResolver for ProcMountinfoResolver {
    fn resolve_mounts(&self, devnode: &Path) -> Result<Vec<PathBuf>, UsbError> {
        let rdev = std::fs::metadata(devnode)
            .map_err(|e| UsbError::AnyError(format!("metadata({}): {e}", devnode.display())))?
            .rdev();

        let want_major = major(rdev);
        let want_minor = minor(rdev);

        let file = File::open("/proc/self/mountinfo")
            .map_err(|e| UsbError::AnyError(format!("open mountinfo: {e}")))?;
        let reader = BufReader::new(file);

        let mut mounts = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| UsbError::AnyError(format!("read mountinfo: {e}")))?;

            let mut parts = line.split(" - ");
            let left = parts.next().unwrap_or("");

            let mut f = left.split_whitespace();
            let _mount_id = f.next();
            let _parent_id = f.next();
            let maj_min = f.next(); 
            let _root = f.next();
            let mount_point = f.next();

            if let (Some(mm), Some(mp)) = (maj_min, mount_point) {
                if let Some((maj_s, min_s)) = mm.split_once(':') {
                    if let (Ok(maj), Ok(min)) = (maj_s.parse::<u64>(), min_s.parse::<u64>()) {
                        if maj == want_major && min == want_minor {
                            mounts.push(PathBuf::from(mp));
                        }
                    }
                }
            }
        }
        Ok(mounts)
    }
}
pub struct RecursiveVideoFinder {
    allowed: HashSet<&'static str>,
    max_depth: Option<usize>,
    follow_links: bool,
}

impl RecursiveVideoFinder {
    pub fn new() -> Self {

        let allowed = HashSet::from([
            "mp4", "mov", "m4v", "mkv", "avi", "wmv", "flv", "webm",
            "mts", "m2ts", "ts",
        ]);
        Self { allowed, max_depth: None, follow_links: false }
    }

    pub fn with_max_depth(mut self, d: usize) -> Self {
        self.max_depth = Some(d);
        self
    }

    pub fn follow_links(mut self, yes: bool) -> Self {
        self.follow_links = yes;
        self
    }

    fn is_video(&self, entry: &DirEntry) -> bool {
        if !entry.file_type().is_file() {
            return false;
        }
        match entry.path().extension().and_then(|s| s.to_str()) {
            Some(ext) => self.allowed.contains(&ext.to_ascii_lowercase()[..]),
            None => false,
        }
    }
}

impl VideoFinder for RecursiveVideoFinder {
    fn find_iter<'a>(&'a self, root: &'a Path)
        -> Box<dyn Iterator<Item = Result<PathBuf, UsbError>> + 'a + Send>
    {
        let mut walk = WalkDir::new(root).follow_links(self.follow_links);
        if let Some(d) = self.max_depth { walk = walk.max_depth(d); }


        let it = walk.into_iter()
            .filter_map(|res| {
                match res {
                    Ok(entry) => {
                        if self.is_video(&entry) {
                            Some(Ok(entry.into_path()))
                        } else {
                            None
                        }
                    }
                    Err(e) => {
             
                        Some(Err(UsbError::AnyError(format!("walk error: {e}"))))
                    }
                }
            });

        Box::new(it)
    }
}