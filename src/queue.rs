use libc::{c_int, size_t, c_long, ssize_t};
use std::default::Default;
use std::io::Error;
use std::cmp::min;

use queue::QueueOperation::Remove;

const BUFFER_SIZE: usize = 2048;
pub const IPC_PRIVATE: i32 = 0;

#[repr(C)]
struct MsgBuf {
    mtype: c_long,
    mtext: [u8; BUFFER_SIZE],
}

bitflags! {
    pub struct NewMessageQueueFlags: c_int {
        const IPC_CREAT = 0o1000;
        const IPC_EXCL  = 0o2000;

        const USER_READ     = 0o400;
        const USER_WRITE    = 0o200;
        const USER_EXECUTE  = 0o100;
        const GROUP_READ    = 0o040;
        const GROUP_WRITE   = 0o020;
        const GROUP_EXECUTE = 0o010;
        const OTHER_READ    = 0o004;
        const OTHER_WRITE   = 0o002;
        const OTHER_EXECUTE = 0o001;

        const USER_RWX  = Self::USER_READ.bits | Self::USER_WRITE.bits | Self::USER_EXECUTE.bits;
        const GROUP_RWX = Self::GROUP_READ.bits | Self::GROUP_WRITE.bits | Self::GROUP_EXECUTE.bits;
        const OTHER_RWX = Self::OTHER_READ.bits | Self::OTHER_WRITE.bits | Self::OTHER_EXECUTE.bits;
    }
}

pub enum QueueOperation {
    Remove = 0,
    Set    = 1,
    Status = 2,
}

impl Default for NewMessageQueueFlags {
    fn default() -> NewMessageQueueFlags { NewMessageQueueFlags::IPC_CREAT | NewMessageQueueFlags::USER_RWX }
}

bitflags! {
    pub struct SendReceiveFlags: c_int {
        const IPC_NOWAIT  = 0o04000;
        const MSG_NOERROR = 0o10000;
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
    pub fn new(id: i32, flags: NewMessageQueueFlags) -> Result<MessageQueue, Error> {
        match unsafe { msgget(id as i32, flags.bits) } {
            -1 => Err(Error::last_os_error()),
            msqid => Ok(MessageQueue{msqid: msqid})
        }
    }

    /// Send a message to the queue
    pub fn send(&self, msg_type: i32, message: &[u8], flags: SendReceiveFlags) -> Result<(), Error> {
        assert!(message.len() <= BUFFER_SIZE);
        let mut buffer = MsgBuf{mtype: msg_type as c_long, mtext: [0; BUFFER_SIZE]};

        for i in 0..min(buffer.mtext.len(), message.len()) {
            buffer.mtext[i] = message[i];
        }

        match unsafe { msgsnd(self.msqid, &mut buffer, message.len() as size_t, flags.bits) } {
            -1 => Err(Error::last_os_error()),
            _ => Ok(())
        }
    }

    /// Receive a message from the queue
    pub fn receive(&self, msg_type: i32, flags: SendReceiveFlags) -> Result<(i32, Vec<u8>), Error> {
        let mut buffer = MsgBuf{mtype: 0, mtext: [0; BUFFER_SIZE]};

        match unsafe { msgrcv(self.msqid, &mut buffer, BUFFER_SIZE as size_t, msg_type as c_long, flags.bits) } {
            -1 => Err(Error::last_os_error()),
            size => {
                assert!(size >= 0);
                let mut result = Vec::new();
                result.extend_from_slice(&buffer.mtext[0..size as usize]);
                Ok((buffer.mtype as i32, result))
            }
        }
    }

    /// Delete the queue
    pub fn remove(self) -> Result<(), Error> {
        match unsafe { msgctl(self.msqid, Remove as c_int, 0 as *mut u8) } {
            -1 => Err(Error::last_os_error()),
            _ => Ok(())
        }
    }
}
