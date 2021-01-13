use crate::{to_io, BlckExt};
use std::fs::File;
use std::io::Result;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::io::AsRawFd;

impl BlckExt for File {
    fn is_block_device(&self) -> bool {
        match self.metadata() {
            Err(_) => false,
            Ok(meta) => meta.file_type().is_block_device(),
        }
    }

    fn get_block_device_size(&self) -> Result<u64> {
        let fd = self.as_raw_fd();
        let mut blksize = 0;
        unsafe {
            ioctls::blkgetsize64(fd, &mut blksize).map_err(to_io)?;
            Ok(blksize)
        }
    }

    fn get_size_of_block(&self) -> Result<u64> {
        let fd = self.as_raw_fd();
        let mut blksize = 0;
        unsafe {
            ioctls::blksszget(fd, &mut blksize).map_err(to_io)?;
        }
        Ok(blksize as u64)
    }

    fn get_block_count(&self) -> Result<u64> {
        Ok(self.get_block_device_size()? / self.get_size_of_block()?)
    }

    fn block_reread_paritions(&self) -> Result<()> {
        let fd = self.as_raw_fd();
        unsafe {
            ioctls::blkrrpart(fd).map_err(to_io)?;
        }
        Ok(())
    }

    fn block_discard_zeros(&self) -> Result<bool> {
        let fd = self.as_raw_fd();
        let mut discard_zeros = 0;
        unsafe {
            ioctls::blkdiscardzeros(fd, &mut discard_zeros).map_err(to_io)?;
        }
        Ok(discard_zeros != 0)
    }

    fn block_discard(&self, offset: u64, len: u64) -> Result<()> {
        let fd = self.as_raw_fd();
        let range = [offset, len];
        unsafe {
            ioctls::blkdiscard(fd, &range).map_err(to_io)?;
        }
        Ok(())
    }

    fn block_zero_out(&mut self, offset: u64, len: u64) -> Result<()> {
        let fd = self.as_raw_fd();
        let range = [offset, len];
        unsafe {
            ioctls::blkzeroout(fd, &range).map_err(to_io)?;
        }
        Ok(())
    }
}

mod ioctls {
    use nix::{
        ioctl_none, ioctl_read_bad, ioctl_write_ptr_bad, request_code_none, request_code_read,
    };

    ioctl_none!(blkrrpart, 0x12, 95);
    ioctl_read_bad!(
        blkgetsize64,
        request_code_read!(0x12, 114, ::std::mem::size_of::<usize>()),
        u64
    );
    ioctl_read_bad!(
        blkdiscardzeros,
        request_code_none!(0x12, 124),
        ::std::os::raw::c_uint
    );
    ioctl_write_ptr_bad!(blkdiscard, request_code_none!(0x12, 119), [u64; 2]);
    ioctl_write_ptr_bad!(blkzeroout, request_code_none!(0x12, 127), [u64; 2]);
    ioctl_read_bad!(
        blksszget,
        request_code_none!(0x12, 104),
        ::std::os::raw::c_int
    );
}
