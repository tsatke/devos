use crate::fcntl::{Process, ProcessAccess};
use crate::UserspaceMutPtr;
use core::slice::from_raw_parts_mut;
use kernel_abi::{Errno, EINVAL, ERANGE};

pub fn sys_getcwd<Cx: ProcessAccess>(
    cx: &Cx,
    buf: UserspaceMutPtr<u8>,
    size: usize,
) -> Result<usize, Errno> {
    if buf.is_null() {
        return Err(EINVAL);
    }
    if size == 0 {
        return Err(EINVAL);
    }

    let slice = unsafe { from_raw_parts_mut(*buf, size) };

    let cwd = cx.current_process().current_dir();
    let guard = cwd.read();
    let bytelen = guard.len();
    if size <= bytelen {
        return Err(ERANGE);
    }
    slice.iter_mut().zip(guard.bytes()).for_each(|(s, b)| {
        *s = b;
    });
    slice[bytelen] = 0; // Null-terminate the string

    Ok(buf.addr())
}

#[cfg(test)]
mod tests {
    use crate::unistd::sys_getcwd;
    use alloc::vec;
    use kernel_abi::{EINVAL, ERANGE};
    use kernel_vfs::path::AbsoluteOwnedPath;
    use spin::rwlock::RwLock;

    #[test]
    fn test_getcwd() {
        struct GetcwdProcess<'a>(&'a RwLock<AbsoluteOwnedPath>);
        impl super::Process for GetcwdProcess<'_> {
            fn current_dir(&self) -> &RwLock<AbsoluteOwnedPath> {
                self.0
            }
        }
        struct GetcwdProcessAccess<'a>(GetcwdProcess<'a>);
        impl<'a> super::ProcessAccess for GetcwdProcessAccess<'a> {
            type Process = GetcwdProcess<'a>;

            fn current_process(&self) -> &Self::Process {
                &self.0
            }
        }

        for args in [
            (("/test/path", 0), Err(EINVAL)),
            (("/test/path", 10), Err(ERANGE)),
            (("/test/path", 11), Ok(())),
        ] {
            let ((path, size), expected) = args;
            let cwd = AbsoluteOwnedPath::try_from(path).unwrap().into();
            let access = GetcwdProcessAccess(GetcwdProcess(&cwd));
            let mut buf = vec![0u8; size];
            let ptr = buf.as_mut_ptr();
            let res = sys_getcwd(&access, ptr.into(), buf.len());
            match expected {
                Ok(()) => match res {
                    Ok(addr) => {
                        assert_eq!(addr, ptr as usize);
                        assert_eq!(path.as_bytes(), &buf[..path.len()]);
                        assert_eq!(0, buf[path.len()]);
                    }
                    Err(e) => panic!("failed with {e} but expected success"),
                },
                Err(e) => {
                    assert_eq!(res, Err(e));
                }
            }
        }
    }
}
