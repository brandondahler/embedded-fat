use crate::queued_resource::QueuedResource;
use core::ops::{Deref, DerefMut};

pub struct QueuedResourceLease<'a, T, const N: usize> {
    queued_resource: &'a QueuedResource<T, N>,
}

impl<'a, T, const N: usize> QueuedResourceLease<'a, T, N> {
    pub(super) fn new(queued_resource: &'a QueuedResource<T, N>) -> Self {
        Self { queued_resource }
    }
}

impl<T, const N: usize> Deref for QueuedResourceLease<'_, T, N> {
    type Target = T;

    /// Dereferences this lease to the underlying resource.
    ///
    /// # Safety
    /// This maintains the required aliasing semantics of the underlying resource by requiring that
    /// the [QueuedResourceLease] reference is itself a reference that follows the standard aliasing
    /// rules.
    fn deref(&self) -> &T {
        unsafe { self.queued_resource.get_resource() }
    }
}

impl<T, const N: usize> DerefMut for QueuedResourceLease<'_, T, N> {
    /// Mutably dereferences this lease to the underlying resource.
    ///
    /// # Safety
    /// This maintains the required aliasing semantics of the underlying resource by requiring that
    /// the [QueuedResourceLease] reference is itself a mutable reference that follows the standard
    /// aliasing rules.
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.queued_resource.get_resource_mut() }
    }
}

impl<T, const N: usize> Drop for QueuedResourceLease<'_, T, N> {
    fn drop(&mut self) {
        self.queued_resource.lease_dropped();
    }
}
