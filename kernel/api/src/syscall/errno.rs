use derive_more::Deref;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct PositiveOrZeroIsize(isize);

impl PositiveOrZeroIsize {
    pub const fn new(value: isize) -> Option<Self> {
        if value < 0 {
            None
        } else {
            Some(Self(value))
        }
    }

    pub fn get(self) -> isize {
        self.0
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct NegativeIsize(isize);

impl NegativeIsize {
    pub const fn new(value: isize) -> Option<Self> {
        if value >= 0 {
            None
        } else {
            Some(Self(value))
        }
    }

    pub fn get(self) -> isize {
        self.0
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Deref)]
pub struct Errno(isize);

impl From<()> for Errno {
    fn from(_: ()) -> Self {
        Self::new(0)
    }
}

impl<T: Into<Errno>, E: Into<Errno>> From<Result<T, E>> for Errno {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(v) => v.into(),
            Err(e) => e.into(),
        }
    }
}

impl From<isize> for Errno {
    fn from(value: isize) -> Self {
        Self(value)
    }
}

impl From<usize> for Errno {
    fn from(value: usize) -> Self {
        match TryInto::<isize>::try_into(value) {
            Ok(v) => Self::new(v),
            Err(_) => Self::EINVAL,
        }
    }
}

impl Errno {
    /// Operation not permitted
    ///
    /// An attempt was made to perform an operation that the caller does not have the required permissions to perform.
    pub const EPERM: Self = Self::new(-1);

    /// No such file or directory
    ///
    /// A component of a specified pathname did not exist, or the pathname was an empty string.
    pub const ENOENT: Self = Self::new(-2);

    /// No such process
    ///
    /// A specified process does not exist, or the process group ID does not match any existing process or process group.
    pub const ESRCH: Self = Self::new(-3);

    /// Interrupted system call
    ///
    /// A system call was interrupted by a signal before it could complete.
    pub const EINTR: Self = Self::new(-4);

    /// Input/output error
    ///
    /// An error occurred while performing an input or output operation on a device or file.
    pub const EIO: Self = Self::new(-5);

    /// No such device or address
    ///
    /// The specified device or address does not exist or is not accessible.
    pub const ENXIO: Self = Self::new(-6);

    /// Argument list too long
    ///
    /// The number of arguments or the total length of the arguments for a command or system call exceeded the maximum allowed size.
    pub const E2BIG: Self = Self::new(-7);

    /// Exec format error
    ///
    /// An executable file has a format error or is not suitable for execution on the current system.
    pub const ENOEXEC: Self = Self::new(-8);

    /// Bad file descriptor
    ///
    /// The specified file descriptor is invalid or not open for the requested operation.
    pub const EBADF: Self = Self::new(-9);

    /// No child processes
    ///
    /// A wait or similar function was called, but there are no child processes to wait for.
    pub const ECHILD: Self = Self::new(-10);

    /// Resource temporarily unavailable (also EAGAIN)
    ///
    /// The requested operation would cause the process to be blocked, and the operation was requested to be non-blocking.
    pub const EWOULDBLOCK: Self = Self::new(-11);

    /// Not enough space (out of memory)
    ///
    /// The system does not have enough memory to complete the requested operation.
    pub const ENOMEM: Self = Self::new(-12);

    /// Permission denied
    ///
    /// The requested operation is not allowed due to insufficient permissions or access rights.
    pub const EACCES: Self = Self::new(-13);

    /// Bad address
    ///
    /// The address specified in a system call or operation is invalid or outside the address space of the process.
    pub const EFAULT: Self = Self::new(-14);

    /// Block device required
    ///
    /// The operation requires a block device, but a non-block device was specified.
    pub const ENOTBLK: Self = Self::new(-15);

    /// Device or resource busy
    ///
    /// The requested resource or device is in use and cannot be accessed or modified at this time.
    pub const EBUSY: Self = Self::new(-16);

    /// File exists
    ///
    /// The specified pathname already exists, and the operation requires that it does not exist.
    pub const EEXIST: Self = Self::new(-17);

    /// Cross-device link
    ///
    /// An attempt was made to create a hard link between files on different filesystems or devices.
    pub const EXDEV: Self = Self::new(-18);

    /// No such device
    ///
    /// The specified device does not exist or is not recognized by the system.
    pub const ENODEV: Self = Self::new(-19);

    /// Not a directory
    ///
    /// A component of the specified pathname exists, but it is not a directory when a directory was expected.
    pub const ENOTDIR: Self = Self::new(-20);

    /// Is a directory
    ///
    /// The specified pathname refers to a directory, but the operation requires a non-directory object.
    pub const EISDIR: Self = Self::new(-21);

    /// Invalid argument
    ///
    /// One or more of the arguments provided to a system call or operation are invalid or out of the acceptable range.
    pub const EINVAL: Self = Self::new(-22);

    /// File table overflow
    ///
    /// The system-wide limit on the total number of open files has been reached.
    pub const ENFILE: Self = Self::new(-23);

    /// Too many open files
    ///
    /// The per-process limit on the number of open file descriptors has been reached.
    pub const EMFILE: Self = Self::new(-24);

    /// Not a typewriter (Inappropriate ioctl for device)
    ///
    /// The specified file descriptor does not refer to a device that supports the requested ioctl operation.
    pub const ENOTTY: Self = Self::new(-25);

    /// Text file busy
    ///
    /// An attempt was made to execute a pure-procedure program that is currently open for writing, or an operation that would modify an executable image is attempted.
    pub const ETXTBSY: Self = Self::new(-26);

    /// File too large
    ///
    /// The size of a file would exceed the maximum file size allowed by the filesystem or the process.
    pub const EFBIG: Self = Self::new(-27);

    /// No space left on device
    ///
    /// There is not enough space left on the device or filesystem to complete the requested operation.
    pub const ENOSPC: Self = Self::new(-28);

    /// Invalid seek
    ///
    /// An attempt was made to seek to an invalid position within a file or device.
    pub const ESPIPE: Self = Self::new(-29);

    /// Read-only file system
    ///
    /// An attempt was made to modify a file or directory on a read-only file system.
    pub const EROFS: Self = Self::new(-30);

    /// Too many links
    ///
    /// An attempt was made to create a new hard link, but the maximum number of hard links for a file has been reached.
    pub const EMLINK: Self = Self::new(-31);

    /// Broken pipe
    ///
    /// A write operation was attempted on a pipe or socket that is not connected or has been closed by the peer.
    pub const EPIPE: Self = Self::new(-32);

    /// Math argument out of domain of function
    ///
    /// A mathematical function was called with an argument outside its domain.
    pub const EDOM: Self = Self::new(-33);

    /// Result too large
    ///
    /// The result of a mathematical operation is too large to be represented within the range of representable values.
    pub const ERANGE: Self = Self::new(-34);

    /// Resource deadlock avoided
    ///
    /// An attempt was made to lock a resource that would have caused a deadlock.
    pub const EDEADLK: Self = Self::new(-35);

    /// File name too long
    ///
    /// A specified pathname or filename is longer than the maximum allowed length.
    pub const ENAMETOOLONG: Self = Self::new(-36);

    /// No locks available
    ///
    /// The system has reached the maximum number of file locks available.
    pub const ENOLCK: Self = Self::new(-37);

    /// Function not implemented
    ///
    /// The requested function or system call is not implemented or not known by the system.
    pub const ENOSYS: Self = Self::new(-38);

    /// Directory not empty
    ///
    /// An attempt was made to remove a directory that is not empty.
    pub const ENOTEMPTY: Self = Self::new(-39);

    /// Too many levels of symbolic links
    ///
    /// The maximum number of symbolic link expansions has been exceeded during the resolution of a pathname.
    pub const ELOOP: Self = Self::new(-40);

    pub const fn new(value: isize) -> Self {
        Self(value)
    }

    pub fn as_isize(self) -> isize {
        *self
    }
}
