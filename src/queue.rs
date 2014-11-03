use libc::{c_int, size_t, c_long, ssize_t};
use std::default::Default;
use std::os;
use std::cmp::min;

const BUFFER_SIZE: uint = 2048;
pub const IPC_PRIVATE: i32 = 0;

#[repr(C)]
struct MsgBuf {
    mtype: c_long,
    mtext: [u8, ..BUFFER_SIZE ],
}

bitflags! {
    flags NewMessageQueueFlags: c_int {
        const IPC_CREAT = 0o1000,
        const IPC_EXCL  = 0o2000,

        const USER_READ     = 0o400,
        const USER_WRITE    = 0o200,
        const USER_EXECUTE  = 0o100,
        const GROUP_READ    = 0o040,
        const GROUP_WRITE   = 0o020,
        const GROUP_EXECUTE = 0o010,
        const OTHER_READ    = 0o004,
        const OTHER_WRITE   = 0o002,
        const OTHER_EXECUTE = 0o001,

        const USER_RWX  = USER_READ.bits | USER_WRITE.bits | USER_EXECUTE.bits,
        const GROUP_RWX = GROUP_READ.bits | GROUP_WRITE.bits | GROUP_EXECUTE.bits,
        const OTHER_RWX = OTHER_READ.bits | OTHER_WRITE.bits | OTHER_EXECUTE.bits,
    }
}

pub enum QueueOperation {
    Remove = 0,
    Set    = 1,
    Status = 2,
}

impl Default for NewMessageQueueFlags {
    fn default() -> NewMessageQueueFlags { IPC_CREAT | USER_RWX }
}

bitflags! {
    flags SendReceiveFlags: c_int {
        const IPC_NOWAIT  = 0o04000,
        const MSG_NOERROR = 0o10000
    }
}

impl Default for SendReceiveFlags {
    fn default() -> SendReceiveFlags { SendReceiveFlags::empty() }
}

extern {
    fn msgget(key: i32, msgflg: c_int) -> c_int;
    fn msgrcv(msqid: c_int, msgp: *mut MsgBuf, msgsz: size_t, msgtyp: c_long, msgflg: c_int) -> ssize_t;
    fn msgsnd(msqid: c_int, msgp: *mut MsgBuf, msgsz: size_t, msgflg: c_int) -> c_int;
    fn msgctl(msqid: c_int, cmd: c_int, buf: *mut u8) -> c_int;

}

pub struct MessageQueue {
    msqid: c_int
}

impl MessageQueue {
    pub fn new(id: i32, flags: NewMessageQueueFlags) -> Result<MessageQueue, String> {
        match unsafe { msgget(id as i32, flags.bits) } {
            -1 => Err(os::last_os_error()),
            msqid => Ok(MessageQueue{msqid: msqid})
        }
    }

    pub fn send(&self, msg_type: int, message: &[u8], flags: SendReceiveFlags) -> Result<(), String> {
        assert!(message.len() <= BUFFER_SIZE);
        let mut buffer = MsgBuf{mtype: msg_type as c_long, mtext: [0, ..BUFFER_SIZE]};

        for i in range(0, min(buffer.mtext.len(), message.len())) {
            buffer.mtext[i] = message[i];
        }

        match unsafe { msgsnd(self.msqid, &mut buffer, message.len() as size_t, flags.bits) } {
            -1 => Err(os::last_os_error()),
            _ => Ok(())
        }
    }

    pub fn receive(&self, msg_type: int, flags: SendReceiveFlags) -> Result<(int, Vec<u8>), String> {
        let mut buffer = MsgBuf{mtype: 0, mtext: [0, ..BUFFER_SIZE]};

        match unsafe { msgrcv(self.msqid, &mut buffer, BUFFER_SIZE as size_t, msg_type as c_long, flags.bits) } {
            -1 => Err(os::last_os_error()),
            size => {
                assert!(size >= 0);
                let mut result = Vec::new();
                result.push_all(buffer.mtext.slice(0, size as uint));
                Ok((buffer.mtype as int, result))
            }
        }
    }

    pub fn remove(self) -> Result<(), String> {
        match unsafe { msgctl(self.msqid, Remove as c_int, 0 as *mut u8) } {
            -1 => Err(os::last_os_error()),
            _ => Ok(())
        }
    }
}
