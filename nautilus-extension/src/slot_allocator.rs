use std::sync::atomic::{AtomicUsize, Ordering};

pub(crate) fn take_next_slot(reserved_slots: &AtomicUsize, max_slots: usize) -> Option<usize> {
    debug_assert!(max_slots <= usize::BITS as usize);

    loop {
        let reserved = reserved_slots.load(Ordering::SeqCst);
        let index = (0..max_slots).find(|index| reserved & slot_mask(*index) == 0)?;
        let next = reserved | slot_mask(index);

        if reserved_slots
            .compare_exchange(reserved, next, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            return Some(index);
        }
    }
}

pub(crate) fn release_slot(reserved_slots: &AtomicUsize, index: usize) {
    if index < usize::BITS as usize {
        reserved_slots.fetch_and(!slot_mask(index), Ordering::SeqCst);
    }
}

pub(crate) fn reset_slots(reserved_slots: &AtomicUsize) {
    reserved_slots.store(0, Ordering::SeqCst);
}

fn slot_mask(index: usize) -> usize {
    1usize << index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocator_reuses_released_slots_without_lifo_ordering() {
        let slots = AtomicUsize::new(0);

        assert_eq!(take_next_slot(&slots, 3), Some(0));
        assert_eq!(take_next_slot(&slots, 3), Some(1));
        assert_eq!(take_next_slot(&slots, 3), Some(2));
        assert_eq!(take_next_slot(&slots, 3), None);

        release_slot(&slots, 1);

        assert_eq!(take_next_slot(&slots, 3), Some(1));
        assert_eq!(take_next_slot(&slots, 3), None);

        release_slot(&slots, 2);
        release_slot(&slots, 0);
        release_slot(&slots, 1);

        assert_eq!(take_next_slot(&slots, 3), Some(0));
        assert_eq!(take_next_slot(&slots, 3), Some(1));
        assert_eq!(take_next_slot(&slots, 3), Some(2));
        assert_eq!(take_next_slot(&slots, 3), None);
    }

    #[test]
    fn release_ignores_out_of_range_indices_and_reset_clears_all_slots() {
        let slots = AtomicUsize::new(0);

        assert_eq!(take_next_slot(&slots, 2), Some(0));
        assert_eq!(take_next_slot(&slots, 2), Some(1));

        release_slot(&slots, usize::BITS as usize);

        assert_eq!(take_next_slot(&slots, 2), None);

        reset_slots(&slots);

        assert_eq!(take_next_slot(&slots, 2), Some(0));
    }

    #[test]
    fn zero_slot_allocator_returns_none() {
        let slots = AtomicUsize::new(0);

        assert_eq!(take_next_slot(&slots, 0), None);
    }
}
