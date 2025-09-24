use uuid::Uuid;
use time::OffsetDateTime;
use crate::usb_domain::{usb_error::*,usb_model::*,};
use std::path::{Path,PathBuf};

pub trait CreateId{
    fn generate_v7_id(&self)->Uuid;
    fn generate_v4_id(&self) ->Uuid;
}

pub trait Clock{
    fn now_local(&self) -> Result<OffsetDateTime,UsbError>;
    fn now_utc(&self) -> Result<OffsetDateTime,UsbError>;
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbEventKind { Added, Removed, Changed }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbEvent {
    pub kind: UsbEventKind,
    pub devnode: Option<PathBuf>,  
    pub info: Option<UsbInfo>,    }

pub trait UsbProbe{
    fn probe_by_devnode(&self,dev:&Path) ->Result<UsbInfo,UsbError>;
    fn list_all(&self) ->Result<Vec<(PathBuf,UsbInfo)>,UsbError>;
}

pub trait MountPathResolver{
    fn resolve_mounts(&self,devnode:&Path) -> Result<Vec<PathBuf>,UsbError>;
}
pub trait UsbEventSource{
    fn watch(&self) ->Result<Box<dyn Iterator<Item = Result<UsbEvent,UsbError>> + Send>,UsbError>;
}
pub trait CreateDir{
    fn ensure_root_dir(&self) -> Result<PathBuf,UsbError>;
    fn ensure_save_folder(&self) -> Result<PathBuf,UsbError>;
}
pub trait SaveToFolder{
    fn save_mp(&self,folder:impl AsRef<Path>) -> Result<PathBuf,UsbError>;
    fn save_snap(&self,folder:impl AsRef<Path>) -> Result<PathBuf,UsbError>;
}

pub trait VideoFinder {
    fn find_iter<'a>(&'a self, root: &'a Path)
        -> Box<dyn Iterator<Item = Result<PathBuf, UsbError>> + 'a + Send>;
    fn find_all(&self, root: &Path) -> Result<Vec<PathBuf>, UsbError> {
        self.find_iter(root).collect()
    }
}