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

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Deref)]
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

impl From<Errno> for isize {
    fn from(value: Errno) -> Self {
        value.as_isize()
    }
}

macro_rules! errnos {
    ($($(#[$($attrs:tt)*])* $name:ident = -$rc:expr),*,) => {
        impl Errno {
            $(
                $(#[$($attrs)*])*
                pub const $name: Self = Self::new(-$rc);
            )*
        }

        impl ::core::fmt::Display for Errno {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let num: isize = (*self).into();
                match num {
                    $(
                    -$rc => write!(f, stringify!($name)),
                    )*
                    x if x < 0 => write!(f, "Unknown({})", num),
                    _ => write!(f, "{}", num),
                }
            }
        }

        impl ::core::fmt::Debug for Errno {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let num: isize = (*self).into();
                match num {
                    $(
                    -$rc => write!(f, "Err({})", stringify!($name)),
                    )*
                    x if x < 0 => write!(f, "Err({})", num),
                    _ => write!(f, "Ok({})", num),
                }
            }
        }
    };
}

errnos! {
    /// Operation not permitted
    ///
    /// An attempt was made to perform an operation that the caller does not have the required permissions to perform.
    EPERM = -1,

    /// No such file or directory
    ///
    /// A component of a specified pathname did not exist, or the pathname was an empty string.
    ENOENT = -2,

    /// No such process
    ///
    /// A specified process does not exist, or the process group ID does not match any existing process or process group.
    ESRCH = -3,

    /// Interrupted system call
    ///
    /// A system call was interrupted by a signal before it could complete.
    EINTR = -4,

    /// Input/output error
    ///
    /// An error occurred while performing an input or output operation on a device or file.
    EIO = -5,

    /// No such device or address
    ///
    /// The specified device or address does not exist or is not accessible.
    ENXIO = -6,

    /// Argument list too long
    ///
    /// The number of arguments or the total length of the arguments for a command or system call exceeded the maximum allowed size.
    E2BIG = -7,

    /// Exec format error
    ///
    /// An executable file has a format error or is not suitable for execution on the current system.
    ENOEXEC = -8,

    /// Bad file descriptor
    ///
    /// The specified file descriptor is invalid or not open for the requested operation.
    EBADF = -9,

    /// No child processes
    ///
    /// A wait or similar function was called, but there are no child processes to wait for.
    ECHILD = -10,

    /// Resource temporarily unavailable (also EAGAIN)
    ///
    /// The requested operation would cause the process to be blocked, and the operation was requested to be non-blocking.
    EWOULDBLOCK = -11,

    /// Not enough space (out of memory)
    ///
    /// The system does not have enough memory to complete the requested operation.
    ENOMEM = -12,

    /// Permission denied
    ///
    /// The requested operation is not allowed due to insufficient permissions or access rights.
    EACCES = -13,

    /// Bad address
    ///
    /// The address specified in a system call or operation is invalid or outside the address space of the process.
    EFAULT = -14,

    /// Block device required
    ///
    /// The operation requires a block device, but a non-block device was specified.
    ENOTBLK = -15,

    /// Device or resource busy
    ///
    /// The requested resource or device is in use and cannot be accessed or modified at this time.
    EBUSY = -16,

    /// File exists
    ///
    /// The specified pathname already exists, and the operation requires that it does not exist.
    EEXIST = -17,

    /// Cross-device link
    ///
    /// An attempt was made to create a hard link between files on different filesystems or devices.
    EXDEV = -18,

    /// No such device
    ///
    /// The specified device does not exist or is not recognized by the system.
    ENODEV = -19,

    /// Not a directory
    ///
    /// A component of the specified pathname exists, but it is not a directory when a directory was expected.
    ENOTDIR = -20,

    /// Is a directory
    ///
    /// The specified pathname refers to a directory, but the operation requires a non-directory object.
    EISDIR = -21,

    /// Invalid argument
    ///
    /// One or more of the arguments provided to a system call or operation are invalid or out of the acceptable range.
    EINVAL = -22,

    /// File table overflow
    ///
    /// The system-wide limit on the total number of open files has been reached.
    ENFILE = -23,

    /// Too many open files
    ///
    /// The per-process limit on the number of open file descriptors has been reached.
    EMFILE = -24,

    /// Not a typewriter (Inappropriate ioctl for device)
    ///
    /// The specified file descriptor does not refer to a device that supports the requested ioctl operation.
    ENOTTY = -25,

    /// Text file busy
    ///
    /// An attempt was made to execute a pure-procedure program that is currently open for writing, or an operation that would modify an executable image is attempted.
    ETXTBSY = -26,

    /// File too large
    ///
    /// The size of a file would exceed the maximum file size allowed by the filesystem or the process.
    EFBIG = -27,

    /// No space left on device
    ///
    /// There is not enough space left on the device or filesystem to complete the requested operation.
    ENOSPC = -28,

    /// Invalid seek
    ///
    /// An attempt was made to seek to an invalid position within a file or device.
    ESPIPE = -29,

    /// Read-only file system
    ///
    /// An attempt was made to modify a file or directory on a read-only file system.
    EROFS = -30,

    /// Too many links
    ///
    /// An attempt was made to create a new hard link, but the maximum number of hard links for a file has been reached.
    EMLINK = -31,

    /// Broken pipe
    ///
    /// A write operation was attempted on a pipe or socket that is not connected or has been closed by the peer.
    EPIPE = -32,

    /// Math argument out of domain of function
    ///
    /// A mathematical function was called with an argument outside its domain.
    EDOM = -33,

    /// Result too large
    ///
    /// The result of a mathematical operation is too large to be represented within the range of representable values.
    ERANGE = -34,

    /// Resource deadlock avoided
    ///
    /// An attempt was made to lock a resource that would have caused a deadlock.
    EDEADLK = -35,

    /// File name too long
    ///
    /// A specified pathname or filename is longer than the maximum allowed length.
    ENAMETOOLONG = -36,

    /// No locks available
    ///
    /// The system has reached the maximum number of file locks available.
    ENOLCK = -37,

    /// Function not implemented
    ///
    /// The requested function or system call is not implemented or not known by the system.
    ENOSYS = -38,

    /// Directory not empty
    ///
    /// An attempt was made to remove a directory that is not empty.
    ENOTEMPTY = -39,

    /// Too many levels of symbolic links
    ///
    /// The maximum number of symbolic link expansions has been exceeded during the resolution of a pathname.
    ELOOP = -40,
}

impl Errno {
    pub const fn new(value: isize) -> Self {
        Self(value)
    }

    pub fn as_isize(self) -> isize {
        *self
    }
}
