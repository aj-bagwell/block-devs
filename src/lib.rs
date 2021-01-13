#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use self::linux::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use self::macos::*;

#[cfg(target_os = "freebsd")]
mod freebsd;
#[cfg(target_os = "freebsd")]
pub use self::freebsd::*;

use std::io::{Result, Seek, SeekFrom, Write};

/// Block device specific extensions to [`File`].
///
/// [`File`]: ../../std/fs/struct.File.html
pub trait BlckExt: Seek + Write {
    /// Test if the file is a block device
    ///
    /// This will return `true` for a block device e.g. `"/dev/sda1"` and `false` for other files
    /// If it returns `false` using the other `BlckExt` methods on this file will almost certainly be an error.
    fn is_block_device(&self) -> bool;

    /// Get the total size of the block device in bytes.
    fn get_block_device_size(&self) -> Result<u64>;

    /// Get the size of one logical blocks in bytes.
    fn get_size_of_block(&self) -> Result<u64>;

    /// Get the number of blocks on the device.
    fn get_block_count(&self) -> Result<u64>;

    /// Ask the OS to re-read the partition table from the device.
    ///
    /// When writing an image to a block device the partions layout may change
    /// this ask the OS to re-read the partion table
    fn block_reread_paritions(&self) -> Result<()>;

    /// Does this device support zeroing on discard.
    ///
    /// Some device (e.g. SSDs with TRIM support) have the ability to mark blocks as unused in a
    /// way that means they will return zeros on future reads.
    ///
    /// If this returns `true` then all calls to [`block_discard`] will cause following reads to return zeros
    ///
    /// Some device only support zeroing on discard for certain sizes and alignements, in which case this
    /// will return `false` but some calls to [`block_discard`] may still result in zeroing some or all of the discared range.
    ///
    /// Since this is a linux only feature other systems will always return false
    ///
    /// [`block_discard`]: #tymethod.block_discard
    fn block_discard_zeros(&self) -> Result<bool>;

    /// Discard a section of the block device.
    ///
    /// Some device e.g. thinly provisioned arrays or SSDs with TRIM support have the ability to mark blocks as unused
    /// to free them up for other use. This may or maynot result in future reads to the discarded section to return
    /// zeros, see [`block_discard_zeros`] for more detail.
    ///
    /// `offset` and `length` should be given in bytes.
    ///
    /// [`block_discard_zeros`]: #tymethod.block_discard_zeros
    fn block_discard(&self, offset: u64, len: u64) -> Result<()>;

    /// Zeros out a section of the block device.
    ///
    /// There is no guaranty that there special kernel support for this and it is unlikely to be
    /// much faster that writing zeros the normal way.
    ///
    /// If there is no system call on a platfrom it will be implement by writing zeros in the normal way
    ///
    /// `offset` and `length` should be given in bytes.
    fn block_zero_out(&mut self, offset: u64, len: u64) -> Result<()> {
        const BUF_SIZE: usize = 1024;
        let zeros = [0; BUF_SIZE];
        let oldpos = self.seek(SeekFrom::Start(offset))?;
        let mut remaining = len;
        while remaining > BUF_SIZE as u64 {
            self.write_all(&zeros)?;
            remaining -= BUF_SIZE as u64;
        }
        self.write_all(&zeros[0..remaining as usize])?;
        self.seek(SeekFrom::Start(oldpos))?;
        Ok(())
    }
}

fn io_error(str: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, str)
}

fn to_io(err: nix::Error) -> std::io::Error {
    match err {
        nix::Error::Sys(errno) => errno.into(),
        nix::Error::InvalidPath => io_error("InvalidPath"),
        nix::Error::InvalidUtf8 => io_error("InvalidUtf8"),
        nix::Error::UnsupportedOperation => io_error("UnsupportedOperation"),
    }
}