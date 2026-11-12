use super::lock_line::{linux_major, linux_minor, proc_locks_held};

fn dev(major: u32, minor: u32) -> u64 {
    ((u64::from(major) & 0xffff_f000) << 32)
        | ((u64::from(major) & 0xfff) << 8)
        | ((u64::from(minor) & 0xffff_ff00) << 12)
        | (u64::from(minor) & 0xff)
}

#[test]
fn proc_locks_match_an_exclusive_flock_only() {
    let device = dev(8, 1);
    assert_eq!(linux_major(device), 8);
    assert_eq!(linux_minor(device), 1);
    let text = "\
1: POSIX ADVISORY WRITE 5433 08:01:7864554 128 128
2: FLOCK ADVISORY READ 2001 08:01:7864554 0 EOF
3: -> FLOCK ADVISORY WRITE 2002 08:01:7864554 0 EOF
4: FLOCK ADVISORY WRITE 2001 08:01:1 0 EOF
5: FLOCK ADVISORY WRITE 2001 08:02:7864554 0 EOF
";
    assert!(!proc_locks_held(text, device, 7_864_554));
    let held = format!("{text}6: FLOCK ADVISORY WRITE 2001 08:01:7864554 0 EOF\n");
    assert!(proc_locks_held(&held, device, 7_864_554));
}
