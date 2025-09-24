use uuid::Uuid;
use time::OffsetDateTime;
use std::path::{Path,PathBuf};
#[derive(PartialEq, Eq,Hash,Clone,Copy)]
pub struct UsbId(Uuid);

impl UsbId {
    fn new(id: Uuid) -> Self { Self(id) }
     fn get(self) -> UsbId { self }   
    fn as_uuid(&self) -> Uuid { self.0 }     
}
#[derive(PartialEq, Eq,Hash,Clone,Debug)]
pub struct UsbInfo{
   pub serial: Option<String>,
   pub vendor: String,
    pub pid: String, 
}
impl UsbInfo{
  pub  fn new(serial:Option<String>,vendor:String,pid:String)->Self{
        Self { serial, vendor, pid }
    }
     fn serial(&self) -> Option<&str> { self.serial.as_deref() }
    fn vendor(&self) -> &str { &self.vendor }
    fn pid(&self) -> &str { &self.pid }
}
#[derive(PartialEq, Eq,Hash,Clone)]
pub struct Usb{
    id : UsbId,
    get_at: OffsetDateTime,
    usb_info:UsbInfo,
}
impl Usb{
    fn new(id:UsbId,get_at:OffsetDateTime,usb_info:UsbInfo)->Self{
        Self { id, get_at, usb_info }
    }
    fn id(&self) -> UsbId {
        self.id 
    }
    fn got_at(&self) -> OffsetDateTime {
        self.get_at 
    }
    fn usb_info(&self) -> &UsbInfo {
        &self.usb_info
    }

}
#[derive(Clone,PartialEq, Eq,Hash,Copy)]
pub struct MpId(Uuid);
impl MpId{
    fn new(id:Uuid) ->Self{
       Self(id)
    }
   fn id(self) -> MpId{
    self
    }
    fn mp_id(&self) -> Uuid{
        self.0
    }
}
#[derive(Clone,PartialEq, Eq,Hash)]
pub struct Mp{
    id:MpId,
    save_folder:PathBuf,
    mp_path:PathBuf,
    snap_path:Option<PathBuf>,
    find_usb:UsbId,
    get_at:OffsetDateTime,
}

impl Mp{
    fn new(id:MpId,save_folder:PathBuf,mp_path:PathBuf,snap_path:Option<PathBuf>,find_usb:UsbId,get_at:OffsetDateTime)->Self{
        Self { id, save_folder, mp_path, snap_path, find_usb, get_at }
    }
 fn find_id(&self) -> MpId { self.id }                  
    fn folder(&self) -> &Path { &self.save_folder }        
    fn mp(&self) -> &Path { &self.mp_path }                 
    fn snap(&self) -> Option<&Path> { self.snap_path.as_deref() } 
    fn which_usb(&self) -> UsbId { self.find_usb }           
    fn when_got(&self) -> OffsetDateTime { self.get_at }    }