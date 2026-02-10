use std::os::fd::{OwnedFd, AsFd, AsRawFd, RawFd};

#[derive(Debug, thiserror::Error)]
pub enum SurfaceflingerError {
    #[error("invalid buffer file descriptor")]
    InvalidBufferFd,
    
    #[error("failed to configure virtual buffer: {0}")]
    BufferConfigurationFailed(anyhow::Error),
    
    #[error("frame notification failed: {0}")]
    FrameNotificationFailed(anyhow::Error),
}

#[derive(Debug)]
pub struct VirtualBuffer {
    fd: OwnedFd,
    width: u32,
    height: u32,
    stride: u32,
    format: u32,
}

impl VirtualBuffer {
    pub fn new(
        fd: OwnedFd,
        width: u32,
        height: u32,
        stride: u32,
        format: u32,
    ) -> Result<Self, SurfaceflingerError> {
        let raw_fd: RawFd = fd.as_fd().as_raw_fd();
        if raw_fd < 0 {
            return Err(SurfaceflingerError::InvalidBufferFd);
        }
        
        Ok(VirtualBuffer {
            fd,
            width,
            height,
            stride,
            format,
        })
    }
    
    pub fn width(&self) -> u32 {
        self.width
    }
    
    pub fn height(&self) -> u32 {
        self.height
    }
    
    pub fn stride(&self) -> u32 {
        self.stride
    }
    
    pub fn format(&self) -> u32 {
        self.format
    }
    
    pub fn into_fd(self) -> OwnedFd {
        self.fd
    }
}

pub trait FrameProvider: Send + Sync {
    fn acquire_frame(&self) -> Result<VirtualBuffer, SurfaceflingerError>;
    fn release_frame(&self, buffer: VirtualBuffer) -> Result<(), SurfaceflingerError>;
    fn on_frame_ready(&self) -> tokio::sync::watch::Receiver<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::fd::{OwnedFd, FromRawFd};
    use nix::fcntl::OFlag;
    use nix::sys::stat::Mode;
    use nix::unistd::close;
    
    #[tokio::test]
    async fn test_virtual_buffer_properties() {
        let fd = nix::fcntl::open(
            "/dev/null",
            OFlag::O_RDWR,
            Mode::empty(),
        ).expect("failed to open /dev/null");
        let owned_fd = unsafe { OwnedFd::from_raw_fd(fd) };
        
        let buffer = VirtualBuffer::new(
            owned_fd,
            1920,
            1080,
            1920,
            0x32315659,
        ).expect("failed to create buffer");
        
        assert_eq!(buffer.width(), 1920);
        assert_eq!(buffer.height(), 1080);
        assert_eq!(buffer.stride(), 1920);
        assert_eq!(buffer.format(), 0x32315659);
    }
    
    #[tokio::test]
    async fn test_virtual_buffer_valid() {
        let fd = nix::fcntl::open(
            "/dev/zero",
            OFlag::O_RDWR,
            Mode::empty(),
        ).expect("failed to open /dev/zero");
        let owned_fd = unsafe { OwnedFd::from_raw_fd(fd) };
        
        let buffer = VirtualBuffer::new(
            owned_fd,
            1920,
            1080,
            1920,
            0x32315659,
        );
        
        assert!(buffer.is_ok());
        let buffer = buffer.unwrap();
        assert_eq!(buffer.width(), 1920);
        assert_eq!(buffer.height(), 1080);
        assert_eq!(buffer.stride(), 1920);
        assert_eq!(buffer.format(), 0x32315659);
    }
}
