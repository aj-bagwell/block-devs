use crate::{io_error, to_io, BlckExt};
use std::fs::File;
use std::io::Result;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::io::AsRawFd;

impl BlckExt for File {
    fn is_block_device(&self) -> bool {
        // free BSD does not support "block" devices, so instead check if file is a disk
        // style device by asking for the block size
        // https://www.freebsd.org/doc/en/books/arch-handbook/driverbasics-block.html
        self.get_size_of_block().is_ok()
    }

    fn get_block_device_size(&self) -> Result<u64> {
        let fd = self.as_raw_fd();
        let mut blksize = 0;
        unsafe {
            ioctls::diocgmediasize(fd, &mut blksize)?;
            Ok(blksize as u64)
        }
    }

    fn get_size_of_block(&self) -> Result<u64> {
        let fd = self.as_raw_fd();
        let mut blksize = 0;
        unsafe {
            ioctls::diocgsectorsize(fd, &mut blksize)?;
        }
        Ok(blksize as u64)
    }

    fn get_block_count(&self) -> Result<u64> {
        Ok(self.get_block_device_size()? / self.get_size_of_block()?)
    }

    fn block_reread_paritions(&self) -> Result<()> {
        Err(crate::Error::UnsupportedOperation)
    }

    fn block_discard_zeros(&self) -> Result<bool> {
        Ok(false)
    }
    fn block_discard(&self, _offset: u64, _len: u64) -> Result<()> {
        Err(crate::Error::UnsupportedOperation)
    }

    fn sync_data(&self) -> Result<()> {
        File::sync_data(self)
    }
}

pub mod ioctls {
    use nix::ioctl_read;

    ioctl_read!(diocgmediasize, b'd', 129, libc::off_t);
    ioctl_read!(diocgsectorsize, b'd', 128, ::std::os::raw::c_uint);
}
