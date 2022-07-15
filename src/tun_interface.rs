
use std::{io::{self, Read, Write}, sync::Arc, fs::File, os::unix::prelude::RawFd};

#[cfg(target_os = "linux")]
extern "C" {
    fn tuntap_setup(fd: libc::c_int, name: *mut u8, mode: libc::c_int, packet_info: libc::c_int) -> libc::c_int;
}

#[cfg(target_os = "macos")]
extern "C" {
    fn tuntap_setup(num: libc::c_uint) -> libc::c_int;
}

#[cfg(target_os = "macos")]
fn get_available_utun() -> Option<u32> {
    use std::{collections::HashSet, process::Command};

    let output = Command::new("ifconfig")
        .args(&["-l"])
        .output()
        .expect("failed to execute ifconfig");
    let interfaces = String::from_utf8_lossy(&output.stdout).into_owned();
    let v = interfaces
        .split([' ', '\n'])
        .filter(|v| v.starts_with("utun"))
        .filter_map(|v| v.replace("utun", "").parse::<u32>().ok())
        .collect::<HashSet<u32>>();

    for i in 0..99 {
        if !v.contains(&i) {
            return Some(i);
        }
    }
    None
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Mode {
    /// TUN mode
    ///
    /// The packets returned are on the IP layer (layer 3), prefixed with 4-byte header (2 bytes
    /// are flags, 2 bytes are the protocol inside, eg one of
    /// <https://en.wikipedia.org/wiki/EtherType#Examples>.
    Tun = 1,
    /// TAP mode
    ///
    /// The packets are on the transport layer (layer 2), and start with ethernet frame header.
    Tap = 2,
}

pub struct TunInterface {
    #[cfg(target_os = "macos")]
    fd: RawFd,
    #[cfg(target_os = "linux")]
    fd: File,
    name: String,
}

impl TunInterface {
    #[cfg(target_os = "macos")]
    pub fn new() -> Result<Self, io::Error> {
        use std::io::{Error, ErrorKind};

        if let Some(num) = get_available_utun() {
            let result = unsafe { tuntap_setup(num) };
            if result < 0 {
                return Err(io::Error::last_os_error());
            }
            let name = format!("utun{}", num);
            
            Ok(TunInterface { fd: result, name: name })
        } else {
            Err(Error::new(ErrorKind::Other, "No available utun"))
        }
    }

    #[cfg(target_os = "linux")]
    pub fn new() -> Result<Self, io::Error> {
        use std::ffi::CStr;
        use std::io::Error;
        use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};
        use std::fs::OpenOptions;

        let fd = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/net/tun")?;
        // The buffer is larger than needed, but who cares… it is large enough.
        let ifname = "";
        let mut name_buffer = Vec::new();
        name_buffer.extend_from_slice(ifname.as_bytes());
        name_buffer.extend_from_slice(&[0; 33]);
        let name_ptr: *mut u8 = name_buffer.as_mut_ptr();
        let result = unsafe {
            tuntap_setup(
                fd.as_raw_fd(),
                name_ptr,
                Mode::Tun as libc::c_int,
                if false { 1 } else { 0 },
            )
        };
        if result < 0 {
            return Err(Error::last_os_error());
        }
        let name = unsafe {
            CStr::from_ptr(name_ptr as *const libc::c_char)
                .to_string_lossy()
                .into_owned()
        };
        Ok(TunInterface { fd, name })
    }

    pub fn name(&self)->&str {
        &self.name
    }
    

    #[cfg(target_os = "linux")]
    pub fn recv(&self, buf: &mut [u8]) -> Result<usize, io::Error> {
        (&self.fd).read(buf)
    }
    #[cfg(target_os = "linux")]
    pub fn send(&self, buf: &[u8]) -> Result<usize, io::Error> {
        (&self.fd).write(buf)
    }

    #[cfg(target_os = "macos")]
    pub fn recv(&self, buf: &mut [u8]) -> Result<usize, io::Error> {
        unsafe {
            let amount = libc::read(self.fd, buf.as_mut_ptr() as *mut _, buf.len());

            if amount < 0 {
                return Err(io::Error::last_os_error().into());
            }

            Ok(amount as usize)
        }
    }
    #[cfg(target_os = "macos")]
    pub fn send(&self, buf: &[u8]) -> Result<usize, io::Error> {
        unsafe {
            let amount = libc::write(self.fd, buf.as_ptr() as *const _, buf.len());

            if amount < 0 {
                return Err(io::Error::last_os_error().into());
            }

            Ok(amount as usize)
        }
    }

}
