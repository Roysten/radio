use std::ffi::c_void;
use std::ffi::CStr;
use std::ffi::CString;

use libc::c_char;
use libc::c_double;
use libc::c_int;

#[derive(Debug, PartialEq, Eq)]
#[repr(C)]
#[allow(dead_code)]
pub enum MpvError {
    Success = 0,
    EventQueueFull = -1,
    NoMem = -2,
    Uninitialized = -3,
    InvalidParameter = -4,
    OptionNotFound = -5,
    OptionFormat = -6,
    OptionError = -7,
    PropertyNotFound = -8,
    PropertyFormat = -9,
    PropertyUnavailable = -10,
    PropertyError = -11,
    Command = -12,
    LoadingFailed = -13,
    AOInitFailed = -14,
    VOInitFailed = -15,
    NothingToPlay = -16,
    UnknownFormat = -17,
    Unsupported = -18,
    NotImplemented = -19,
    Generic = -20,
    /**
     * Error added by the Rust wrapper. If this ever gets returned it's a BUG!
     */
    Fatal = -1337,
}

#[derive(Copy, Clone)]
#[repr(C)]
#[allow(dead_code)]
pub enum MpvFormat {
    None = 0,
    String = 1,
    OsdString = 2,
    Flag = 3,
    Int64 = 4,
    Double = 5,
    Node = 6,
    NodeArray = 7,
    NodeMap = 8,
    ByteArray = 9,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
#[allow(dead_code)]
pub enum CMpvEventId {
    None = 0,
    Shutdown = 1,
    LogMessage = 2,
    GetPropertyReply = 3,
    SetPropertyReply = 4,
    CommandReply = 5,
    StartFile = 6,
    EndFile = 7,
    FileLoaded = 8,
    TracksChanged = 9,
    TrackSwitched = 10,
    Idle = 11,
    Pause = 12,
    Unpause = 13,
    Tick = 14,
    ScriptInputDispatch = 15,
    ClientMessage = 16,
    VideoReconfig = 17,
    AudioReconfig = 18,
    MetadataUpdate = 19,
    Seek = 20,
    PlaybackRestart = 21,
    PropertyChange = 22,
    ChapterChange = 23,
    QueueOverflow = 24,
    Hook = 25,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct CMpvEvent {
    event_id: CMpvEventId,
    error: c_int,
    reply_userdata: u64,
    data: *mut c_void,
}

#[repr(C)]
struct CMpvEventProperty {
    name: *const c_char,
    format: MpvFormat,
    data: *const c_void,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum MpvEvent {
    None,
    Shutdown,
    LogMessage,
    GetPropertyReply,
    SetPropertyReply,
    CommandReply,
    StartFile,
    EndFile,
    FileLoaded,
    TracksChanged,
    TrackSwitched,
    Idle,
    Pause,
    Unpause,
    Tick,
    ScriptInputDispatch,
    ClientMessage,
    VideoReconfig,
    AudioReconfig,
    MetadataUpdate,
    Seek,
    PlaybackRestart,
    PropertyChange {
        name: String,
        change: String,
        reply_userdata: u64,
    },
    ChapterChange,
    QueueOverflow,
    Hook,
}

impl From<CMpvEvent> for MpvEvent {
    fn from(event: CMpvEvent) -> Self {
        match event.event_id {
            CMpvEventId::None => MpvEvent::None,
            CMpvEventId::Shutdown => MpvEvent::Shutdown,
            CMpvEventId::LogMessage => MpvEvent::LogMessage,
            CMpvEventId::GetPropertyReply => MpvEvent::GetPropertyReply,
            CMpvEventId::SetPropertyReply => MpvEvent::SetPropertyReply,
            CMpvEventId::CommandReply => MpvEvent::CommandReply,
            CMpvEventId::StartFile => MpvEvent::StartFile,
            CMpvEventId::EndFile => MpvEvent::EndFile,
            CMpvEventId::FileLoaded => MpvEvent::FileLoaded,
            CMpvEventId::TracksChanged => MpvEvent::TracksChanged,
            CMpvEventId::TrackSwitched => MpvEvent::TrackSwitched,
            CMpvEventId::Idle => MpvEvent::Idle,
            CMpvEventId::Pause => MpvEvent::Pause,
            CMpvEventId::Unpause => MpvEvent::Unpause,
            CMpvEventId::Tick => MpvEvent::Tick,
            CMpvEventId::ScriptInputDispatch => MpvEvent::ScriptInputDispatch,
            CMpvEventId::ClientMessage => MpvEvent::ClientMessage,
            CMpvEventId::VideoReconfig => MpvEvent::VideoReconfig,
            CMpvEventId::AudioReconfig => MpvEvent::AudioReconfig,
            CMpvEventId::MetadataUpdate => MpvEvent::MetadataUpdate,
            CMpvEventId::Seek => MpvEvent::Seek,
            CMpvEventId::PlaybackRestart => MpvEvent::PlaybackRestart,
            CMpvEventId::PropertyChange => unsafe {
                let property = &*(event.data as *const CMpvEventProperty);
                let txt = *(property.data as *const *const c_char);
                MpvEvent::PropertyChange {
                    name: CStr::from_ptr(property.name).to_str().unwrap().to_string(),
                    change: CStr::from_ptr(txt).to_str().unwrap().to_string(),
                    reply_userdata: event.reply_userdata,
                }
            },
            CMpvEventId::ChapterChange => MpvEvent::ChapterChange,
            CMpvEventId::QueueOverflow => MpvEvent::QueueOverflow,
            CMpvEventId::Hook => MpvEvent::Hook,
        }
    }
}

extern "C" {

    fn mpv_create() -> *mut c_void;

    fn mpv_initialize(ctx: *mut c_void) -> MpvError;

    fn mpv_destroy(ctx: *mut c_void);

    fn mpv_command(ctx: *mut c_void, args: *const *const c_char) -> MpvError;

    fn mpv_wait_event(ctx: *mut c_void, timeout: c_double) -> *const CMpvEvent;

    fn mpv_observe_property(
        ctx: *mut c_void,
        reply_userdata: u64,
        name: *const c_char,
        format: MpvFormat,
    ) -> MpvError;

}

pub struct MpvCtx {
    ctx: *mut c_void,
}

impl<'a> MpvCtx {
    pub fn create() -> Result<MpvCtx, (&'static str)> {
        let ctx = unsafe { mpv_create() };
        if ctx.is_null() {
            Err("Intialization of context failed")
        } else {
            Ok(MpvCtx { ctx })
        }
    }

    pub fn init(&mut self) -> Result<(), MpvError> {
        let init_status = unsafe { mpv_initialize(self.ctx) };
        if init_status == MpvError::Success {
            Ok(())
        } else {
            Err(init_status)
        }
    }

    pub fn command(&mut self, args: &[&'a str]) -> Result<(), MpvError> {
        let c_args = args
            .iter()
            .map(|&x| CString::new(x).expect("Failed to convert string slice to C string"))
            .collect::<Vec<CString>>();

        let mut c_args_ptrs = c_args
            .iter()
            .map(|ref x| x.as_ptr())
            .collect::<Vec<*const c_char>>();
        c_args_ptrs.push(std::ptr::null::<c_char>());

        let result = unsafe { mpv_command(self.ctx, c_args_ptrs.as_ptr()) };
        if result == MpvError::Success {
            Ok(())
        } else {
            Err(MpvError::from(result))
        }
    }

    pub fn wait_event(&mut self, timeout: f64) -> Result<MpvEvent, MpvError> {
        let event = unsafe { mpv_wait_event(self.ctx, timeout) };
        if event.is_null() {
            panic!("The return value of mpv_wait_event was NULL, which, according to the docs, is not possible.");
        }

        let error = unsafe { std::mem::transmute((*event).error) };
        if error != MpvError::Success {
            Err(error)
        } else {
            Ok(unsafe { MpvEvent::from(*event) })
        }
    }

    pub fn observe_property(
        &mut self,
        reply_userdata: u64,
        name: &str,
        format: MpvFormat,
    ) -> Result<(), MpvError> {
        let name = CString::new(name).expect("Failed to convert string slice to C string");
        let result =
            unsafe { mpv_observe_property(self.ctx, reply_userdata, name.as_ptr(), format) };
        if result == MpvError::Success {
            Ok(())
        } else {
            Err(result)
        }
    }
}

impl Drop for MpvCtx {
    fn drop(&mut self) {
        unsafe { mpv_destroy(self.ctx) }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_wait_event() {
        let mut ctx = MpvCtx::create().expect("Creating context failed");
        ctx.init().expect("Failed to initialize context");
        ctx.observe_property(0, "metadata", MpvFormat::MpvFormatString)
            .expect("Cannot observe metadata property");
        ctx.command(&["loadfile", "http://stream.gal.io/arrow"])
            .expect("Error opening URL");

        let mut count = 0;
        loop {
            let result = ctx.wait_event(-1.0);
            if let Ok(event) = result {
                match event {
                    MpvEvent::PropertyChange {
                        name,
                        change,
                        reply_userdata,
                    } => {
                        println!("{} {} {}", name, change, reply_userdata);
                    }
                    _ => println!("Received event: {:?}", event),
                }
            } else {
                println!("Wait event encountered an error event");
            }
            println!("{}", count);
            count += 1;
        }
    }

}
