use rustix::{
    fd::{AsFd, OwnedFd},
    fs::{self, FlockOperation, Mode, OFlags},
};
use std::{io, path::Path};

/// A RAII file lock.
#[derive(Debug)]
pub struct Flock {
    fd: OwnedFd,
}

impl Flock {
    /// Acquire an exclusive lock on a file.
    ///
    /// If the file does not exist, it will be created.
    /// The lock is released when the returned [`Flock`] is dropped.
    pub fn lock(path: &Path) -> io::Result<Self> {
        let fd = fs::openat(
            fs::CWD,
            path,
            OFlags::CREATE | OFlags::WRONLY,
            Mode::RUSR | Mode::WUSR,
        )
        .map_err(|e| io::Error::from_raw_os_error(e.raw_os_error()))?;

        fs::flock(fd.as_fd(), FlockOperation::NonBlockingLockExclusive)
            .map_err(|_| io::Error::new(io::ErrorKind::AddrInUse, "Lock already held"))?;

        Ok(Self { fd })
    }
}

impl Drop for Flock {
    fn drop(&mut self) {
        let _ = fs::flock(self.fd.as_fd(), FlockOperation::Unlock);
    }
}
