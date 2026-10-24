use core::task::Waker;

pub struct DeviceWakerQueueLease(usize);

enum State {
    Waiting(Waker),
    Running,
}

#[derive(Clone, Debug)]
pub struct DeviceWakerQueue<const N: usize> {
    wakers: [Option<Waker>; N],
    start: usize,
    size: usize,
}

impl<const N: usize> DeviceWakerQueue<N> {
    pub fn new() -> DeviceWakerQueue<N> {
        Self {
            wakers: [const { None }; N],
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
        if (self.size >= N) {
            return None;
        }

        let insert_index = Self::wrapping_add(self.start, self.size);
        self.wakers[insert_index] = Some(waker);
        self.size += 1;

        Some(DeviceWakerQueueLease(insert_index))
    }

    /// Pops from the front the next non-canceled waker.
    ///
    /// A value of `None` implies that the queue is now empty.
    pub fn pop(&mut self) -> Option<Waker> {
        while (self.size > 0) {
            let current_index = self.start;
            self.start = Self::wrapping_add(self.start, 1);
            self.size -= 1;

            if let Some(next_waker) = self.wakers[current_index].take() {
                return Some(next_waker);
            }
        }

        None
    }

    pub fn try_update(&mut self, lease: &DeviceWakerQueueLease, waker: Waker) -> bool {
        match self.wakers[lease.0].as_mut() {
            Some(current_waker) => {
                *current_waker = waker;
                true
            }
            None => false,
        }
    }

    pub fn try_cancel(&mut self, lease: DeviceWakerQueueLease) -> bool {
        self.wakers[lease.0].take().is_some()
    }

    #[inline(always)]
    fn wrapping_add(value: usize, amount: usize) -> usize {
        (value + amount) % N
    }
}
