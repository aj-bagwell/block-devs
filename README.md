# Safe portable wrapper for block device opperations

[![crates.io](http://meritbadge.herokuapp.com/block-devs)](https://crates.io/crates/block-devs)

[Documentation (Releases)](https://docs.rs/block-devs/)

Block Devs provides safe wrappers for the ioctl call for
dealing with block devices (USB sticks, SSDs, hard drives etc).

It aims to provide a consitent interface across all platforms for things like
getting the number of bytes a disk has.

It does this by a extention trait on the standard `File` struct.

```rust,ignore
    use block_devs::BlockExt;
    use std::fs::File;
    
    let path = "/dev/sda2";
    let file = File::open(path)?;
    let count = file.get_block_count().unwrap();
    let bytes = file.get_block_device_size()?;
    let gb = bytes >> 30;

    println!("disk is {} blocks totaling {}gb", count, gb);
```

## Supported Platforms

It currently supports Linux, OS X, and Free BSD, pull requests for other platforms are welcome

## License

block-devs is licensed under the MIT license.  See [LICENSE](LICENSE) for more details.
