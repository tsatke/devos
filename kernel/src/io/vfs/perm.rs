use bitflags::bitflags;

bitflags! {
    pub struct Permission: u16 {
        const STICKY = 1 << 9;

        const USER_READ = 1 << 8;
        const USER_WRITE = 1 << 7;
        const USER_EXECUTE = 1 << 6;

        const GROUP_READ = 1 << 5;
        const GROUP_WRITE = 1 << 4;
        const GROUP_EXECUTE = 1 << 3;

        const OTHER_READ = 1 << 2;
        const OTHER_WRITE = 1 << 1;
        const OTHER_EXECUTE = 1 << 0;
    }
}

impl Permission {
    pub fn user_rwx() -> Self {
        let mut perm = Self::empty();
        perm.set_readable(Triad::User, true);
        perm.set_writable(Triad::User, true);
        perm.set_executable(Triad::User, true);
        perm
    }

    pub fn group_rwx() -> Self {
        let mut perm = Self::empty();
        perm.set_readable(Triad::Group, true);
        perm.set_writable(Triad::Group, true);
        perm.set_executable(Triad::Group, true);
        perm
    }

    pub fn other_rwx() -> Self {
        let mut perm = Self::empty();
        perm.set_readable(Triad::Other, true);
        perm.set_writable(Triad::Other, true);
        perm.set_executable(Triad::Other, true);
        perm
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Triad {
    User,
    Group,
    Other,
}

impl Triad {
    fn get_read_perm(&self) -> Permission {
        match self {
            Triad::User => Permission::USER_READ,
            Triad::Group => Permission::GROUP_READ,
            Triad::Other => Permission::OTHER_READ,
        }
    }

    fn get_write_perm(&self) -> Permission {
        match self {
            Triad::User => Permission::USER_WRITE,
            Triad::Group => Permission::GROUP_WRITE,
            Triad::Other => Permission::OTHER_WRITE,
        }
    }

    fn get_execute_perm(&self) -> Permission {
        match self {
            Triad::User => Permission::USER_EXECUTE,
            Triad::Group => Permission::GROUP_EXECUTE,
            Triad::Other => Permission::OTHER_EXECUTE,
        }
    }
}

impl Default for Permission {
    fn default() -> Self {
        Self::empty()
    }
}

impl Permission {
    pub fn can_read(&self, triad: Triad) -> bool {
        self.contains(triad.get_read_perm())
    }

    pub fn can_write(&self, triad: Triad) -> bool {
        self.contains(triad.get_write_perm())
    }

    pub fn can_execute(&self, triad: Triad) -> bool {
        self.contains(triad.get_execute_perm())
    }

    pub fn is_sticky(&self) -> bool {
        self.contains(Permission::STICKY)
    }

    pub fn set_readable(&mut self, triad: Triad, readable: bool) {
        self.set(triad.get_read_perm(), readable)
    }

    pub fn set_writable(&mut self, triad: Triad, writable: bool) {
        self.set(triad.get_write_perm(), writable)
    }

    pub fn set_executable(&mut self, triad: Triad, executable: bool) {
        self.set(triad.get_execute_perm(), executable)
    }
}
