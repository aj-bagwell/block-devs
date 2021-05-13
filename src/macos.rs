use crate::{io_error, to_io, BlckExt};
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
        Ok(self.get_size_of_block()? * self.get_block_count()?)
    }

    fn get_size_of_block(&self) -> Result<u64> {
        unsafe {
            let fd = self.as_raw_fd();
            let mut blksize: u32 = 0;
            ioctls::dkiocgetblocksize(fd, &mut blksize).map_err(to_io)?;
            Ok(blksize as u64)
        }
    }

    fn get_block_count(&self) -> Result<u64> {
        unsafe {
            let fd = self.as_raw_fd();
            let mut blkcount: u64 = 0;
            ioctls::dkiocgetblockcount(fd, &mut blkcount).map_err(to_io)?;
            Ok(blkcount)
        }
    }

    fn block_reread_paritions(&self) -> Result<()> {
        Err(io_error("UnsupportedOperation"))
    }

    fn block_discard_zeros(&self) -> Result<bool> {
        Ok(false)
    }

    fn block_discard(&self, offset: u64, length: u64) -> Result<()> {
        let fd = self.as_raw_fd();
        let range = [ioctls::dk_extent { offset, length }];
        let unmap = ioctls::dk_unmap::new(&range, 0);
        unsafe {
            ioctls::dkiocunmap(fd, &unmap).map_err(to_io)?;
        }
        Ok(())
    }

    fn sync_data(&self) -> Result<()> {
        File::sync_data(self)
    }
}

#[allow(clippy::missing_safety_doc)]
mod ioctls {
    use nix::{ioctl_read, ioctl_write_ptr};
    use std::marker::PhantomData;

    ioctl_read!(dkiocgetblocksize, b'd', 24, u32);
    ioctl_read!(dkiocgetblockcount, b'd', 25, u64);

    #[repr(C)]
    #[derive(Copy, Clone, Debug, Default)]
    pub struct dk_extent {
        pub offset: u64,
        pub length: u64,
    }
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct dk_unmap<'a> {
        extents: *const dk_extent,
        extents_count: u32,
        pub options: u32,
        phantom: PhantomData<&'a dk_extent>,
    }

    impl<'a> dk_unmap<'a> {
        pub fn new(extents: &'a [dk_extent], options: u32) -> dk_unmap<'a> {
            dk_unmap {
                extents: extents.as_ptr(),
                extents_count: extents.len() as u32,
                options,
                phantom: PhantomData,
            }
        }

        pub fn extents(&'a self) -> &'a [dk_extent] {
            unsafe { std::slice::from_raw_parts(self.extents, self.extents_count as usize) }
        }
    }

    impl ::std::default::Default for dk_unmap<'static> {
        fn default() -> Self {
            unsafe { ::std::mem::zeroed() }
        }
    }

    impl<'a> ::std::fmt::Debug for dk_unmap<'a> {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(
                f,
                "dk_unmap {{ extents: {:?}, options: {} }}",
                self.extents(),
                self.options
            )
        }
    }

    ioctl_write_ptr!(dkiocunmap, b'd', 31, dk_unmap);
}
