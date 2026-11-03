use core::task::Waker;
use core::{debug_assert_matches, mem};

pub struct DeviceWakerQueueLease(usize);

#[derive(Clone, Debug)]
enum WakerSlot {
    Free,
    Waiting(Waker),
    Running,
}

#[derive(Clone, Debug)]
pub struct DeviceWakerQueue<const N: usize> {
    waker_slots: [WakerSlot; N],
    start: usize,
    size: usize,
}

impl<const N: usize> DeviceWakerQueue<N> {
    pub fn new() -> DeviceWakerQueue<N> {
        Self {
            waker_slots: [const { WakerSlot::Free }; N],
            start: 0,
            size: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Pushes the provided waker into the next available slot and returns its lease if space is
    /// available.
    pub fn push(&mut self, waker: Waker) -> Option<DeviceWakerQueueLease> {
        debug_assert!(self.size <= N);

        if (self.size == N) {
            // Queue is full
            return None;
        }

        let insert_index = Self::wrapping_add(self.start, self.size);
        let waker_slot = &mut self.waker_slots[insert_index];

        debug_assert_matches!(waker_slot, WakerSlot::Free);
        *waker_slot = WakerSlot::Waiting(waker);

        self.size += 1;

        Some(DeviceWakerQueueLease(insert_index))
    }

    /// Gets the next non-canceled waker and marks it as ready.
    ///
    /// A value of `None` implies that the queue is now empty.
    pub fn next_waker(&mut self) -> Option<Waker> {
        while (self.size > 0) {
            let current_index = self.start;
            let waker_slot = &mut self.waker_slots[current_index];

            if (matches!(waker_slot, WakerSlot::Free)) {
                self.start = Self::wrapping_add(current_index, 1);
                self.size -= 1;
                continue;
            }

            match mem::replace(waker_slot, WakerSlot::Running) {
                WakerSlot::Waiting(waker) => {
                    return Some(waker);
                }
                WakerSlot::Running => panic!("Waker slot was already marked as running"),
                WakerSlot::Free => unreachable!(),
            }
        }

        None
    }

    /// Updates the held lease if it is still waiting.  
    pub fn try_update(
        &mut self,
        lease: DeviceWakerQueueLease,
        waker: &Waker,
    ) -> Option<DeviceWakerQueueLease> {
        let waker_slot = &mut self.waker_slots[lease.0];

        match waker_slot {
            WakerSlot::Free => panic!("Attempted to update free waker slot"),
            WakerSlot::Waiting(waker_slot_waker) => {
                waker_slot_waker.clone_from(waker);
                Some(lease)
            }
            WakerSlot::Running => {
                *waker_slot = WakerSlot::Free;

                self.start = Self::wrapping_add(self.start, 1);
                self.size -= 1;

                None
            }
        }
    }

    pub fn try_cancel(&mut self, lease: DeviceWakerQueueLease) -> bool {
        let waker_slot = &mut self.waker_slots[lease.0];

        match waker_slot {
            WakerSlot::Free => panic!("Attempted to cancel already free waker slot"),
            WakerSlot::Waiting(waker) => {
                *waker_slot = WakerSlot::Free;
                true
            }
            WakerSlot::Running => false,
        }
    }

    #[inline(always)]
    fn wrapping_add(value: usize, amount: usize) -> usize {
        (value + amount) % N
    }
}
