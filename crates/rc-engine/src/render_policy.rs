//! Pure allocation/cache decisions. No graphics or game dependencies.
pub fn same_pair<T: Eq>(a: T, b: T, x: T, y: T) -> bool {
    (a == x && b == y) || (a == y && b == x)
}

pub fn capacity(current: u64, required: u64) -> u64 {
    if required <= current {
        current
    } else {
        required
            .max(256)
            .checked_next_power_of_two()
            .unwrap_or(required)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ping_pong_does_not_invalidate_identity() {
        assert!(same_pair(1, 2, 1, 2));
        assert!(same_pair(1, 2, 2, 1));
        assert!(!same_pair(1, 2, 1, 3));
    }
    #[test]
    fn allocation_grows_only_and_covers_payload() {
        let mut size = 0;
        for required in [36, 72, 512, 100, 0, 1028, 36] {
            let next = capacity(size, required);
            assert!(next >= size && next >= required);
            if required <= size {
                assert_eq!(size, next);
            }
            size = next;
        }
    }
}
