//! File descriptor / handle budget management.

pub fn get_fd_limit() -> usize {
    #[cfg(unix)]
    {
        let mut rlim = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
        if unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut rlim) } == 0 {
            return rlim.rlim_cur as usize;
        }
    }
    // Default safe budget for Windows and fallback
    8192
}
