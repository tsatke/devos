/// Used for file block counts.
pub type blkcnt_t = i64;
/// Used for block sizes.
pub type blksize_t = u64;
/// Used for system times in clock ticks or CLOCKS_PER_SEC; see <time.h>.
pub type clock_t = u64;
/// Used for clock ID type in the clock and timer functions.
pub type clockid_t = u64;
/// Used for device IDs.
pub type dev_t = u64;
/// Used for file system block counts.
pub type fsblkcnt_t = u64;
/// Used for file system file counts.
pub type fsfilcnt_t = u64;
/// Used for group IDs.
pub type gid_t = u64;
/// Used as a general identifier; can be used to contain at least a pid_t, uid_t, or gid_t.
pub type id_t = u64;
/// Used for file serial numbers.
pub type ino_t = u64;
/// Used for XSI interprocess communication.
pub type key_t = u64;
/// Used for some file attributes.
pub type mode_t = u64;
/// Used for link counts.
pub type nlink_t = u64;
/// Used for file sizes.
pub type off_t = u64;
/// Used for process IDs and process group IDs.
pub type pid_t = i64;
/// Used to identify a thread attribute object.
pub type pthread_attr_t = u64;
/// Used to identify a barrier.
pub type pthread_barrier_t = ();
/// Used to define a barrier attributes object.
pub type pthread_barrierattr_t = ();
/// Used for condition variables.
pub type pthread_cond_t = ();
/// Used to identify a condition attribute object.
pub type pthread_condattr_t = ();
/// Used for thread-specific data keys.
pub type pthread_key_t = ();
/// Used for mutexes.
pub type pthread_mutex_t = ();
/// Used to identify a mutex attribute object.
pub type pthread_mutexattr_t = ();
/// Used for dynamic package initialization.
pub type pthread_once_t = ();
/// Used for read-write locks.
pub type pthread_rwlock_t = ();
/// Used for read-write lock attributes.
pub type pthread_rwlockattr_t = ();
/// Used to identify a spin lock.
pub type pthread_spinlock_t = ();
/// Used to identify a thread.
pub type pthread_t = ();
/// Used for directory entry lengths.
pub type reclen_t = u64;
/// Used for sizes of objects.
pub type size_t = u64;
/// Used for a count of bytes or an error indication.
pub type ssize_t = i64;
/// Used for time in microseconds.
pub type suseconds_t = u64;
/// Used for time in seconds.
pub type time_t = u64;
/// Used for timer ID returned by timer_create().
pub type timer_t = u64;
/// Used for user IDs.
pub type uid_t = u64;
