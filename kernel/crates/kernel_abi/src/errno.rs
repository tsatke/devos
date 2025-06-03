use core::ffi::c_int;

#[derive(Debug)]
#[repr(transparent)]
pub struct Errno(c_int);

impl From<Errno> for isize {
    fn from(errno: Errno) -> Self {
        errno.0 as isize
    }
}

pub const E2BIG: Errno = Errno(1);
pub const EACCES: Errno = Errno(2);
pub const EADDRINUSE: Errno = Errno(3);
pub const EADDRNOTAVAIL: Errno = Errno(4);
pub const EAFNOSUPPORT: Errno = Errno(5);
pub const EAGAIN: Errno = Errno(6);
pub const EALREADY: Errno = Errno(7);
pub const EBADF: Errno = Errno(8);
pub const EBADMSG: Errno = Errno(9);
pub const EBUSY: Errno = Errno(10);
pub const ECANCELED: Errno = Errno(11);
pub const ECHILD: Errno = Errno(12);
pub const ECONNABORTED: Errno = Errno(13);
pub const ECONNREFUSED: Errno = Errno(14);
pub const EDEADLK: Errno = Errno(15);
pub const EDESTADDRREQ: Errno = Errno(16);
pub const EDOM: Errno = Errno(17);
pub const EDQUOT: Errno = Errno(18);
pub const EEXIST: Errno = Errno(19);
pub const EFAULT: Errno = Errno(20);
pub const EFBIG: Errno = Errno(21);
pub const EHOSTUNREACH: Errno = Errno(22);
pub const EIDRM: Errno = Errno(23);
pub const EILSEQ: Errno = Errno(24);
pub const EINPROGRESS: Errno = Errno(25);
pub const EINTR: Errno = Errno(26);
pub const EINVAL: Errno = Errno(27);
pub const EIO: Errno = Errno(28);
pub const EISCONN: Errno = Errno(29);
pub const EISDIR: Errno = Errno(30);
pub const ELOOP: Errno = Errno(31);
pub const EMFILE: Errno = Errno(32);
pub const EMLINK: Errno = Errno(33);
pub const EMSGSIZE: Errno = Errno(34);
pub const EMULTIHOP: Errno = Errno(35);
pub const ENAMETOOLONG: Errno = Errno(36);
pub const ENETDOWN: Errno = Errno(37);
pub const ENETRESET: Errno = Errno(38);
pub const ENETUNREACH: Errno = Errno(39);
pub const ENFILE: Errno = Errno(40);
pub const ENOBUFS: Errno = Errno(41);
pub const ENODEV: Errno = Errno(42);
pub const ENOENT: Errno = Errno(43);
pub const ENOEXEC: Errno = Errno(44);
pub const ENOLCK: Errno = Errno(45);
pub const ENOLINK: Errno = Errno(46);
pub const ENOMEM: Errno = Errno(47);
pub const ENOMSG: Errno = Errno(48);
pub const ENOPROTOOPT: Errno = Errno(49);
pub const ENOSPC: Errno = Errno(50);
pub const ENOSYS: Errno = Errno(51);
pub const ENOTCONN: Errno = Errno(52);
pub const ENOTDIR: Errno = Errno(53);
pub const ENOTEMPTY: Errno = Errno(54);
pub const ENOTRECOVERABLE: Errno = Errno(55);
pub const ENOTSOCK: Errno = Errno(56);
pub const ENOTSUP: Errno = Errno(57);
pub const ENOTTY: Errno = Errno(58);
pub const ENXIO: Errno = Errno(59);
pub const EOPNOTSUPP: Errno = Errno(60);
pub const EOVERFLOW: Errno = Errno(61);
pub const EOWNERDEAD: Errno = Errno(62);
pub const EPERM: Errno = Errno(63);
pub const EPIPE: Errno = Errno(64);
pub const EPROTO: Errno = Errno(65);
pub const EPROTONOSUPPORT: Errno = Errno(66);
pub const EPROTOTYPE: Errno = Errno(67);
pub const ERANGE: Errno = Errno(68);
pub const EROFS: Errno = Errno(69);
pub const ESOCKTNOSUPPORT: Errno = Errno(70);
pub const ESPIPE: Errno = Errno(71);
pub const ESRCH: Errno = Errno(72);
pub const ESTALE: Errno = Errno(73);
pub const ETIMEDEOUT: Errno = Errno(74);
pub const ETXTBSY: Errno = Errno(75);
pub const EWOULDBLOCK: Errno = Errno(76);
pub const EXDEV: Errno = Errno(77);
