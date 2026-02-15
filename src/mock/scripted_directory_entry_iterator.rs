use crate::Device;
use crate::directory_entry::{
    DirectoryEntry, DirectoryEntryIterationError, DirectoryEntryIterator,
    DirectoryEntryIteratorResult,
};
use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::RefCell;
use core::fmt::Debug;
use embedded_io::{ErrorType, SeekFrom};

#[cfg(feature = "sync")]
use {
    crate::device::SyncDevice,
    embedded_io::{Read, Seek},
};

use crate::mock::{CoreError, IoError};
#[cfg(feature = "async")]
use {
    crate::AsyncDevice,
    embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek},
};

#[derive(Clone)]
pub struct ScriptedDirectoryEntryIterator<'a, D>
where
    D: Device,
{
    call_index: usize,

    peek: Rc<dyn Fn(usize) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> + 'a>,
    advance: Rc<dyn Fn(usize) -> DirectoryEntryIteratorResult<bool, D> + 'a>,
    next: Rc<dyn Fn(usize) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> + 'a>,
}

impl<'a, D> ScriptedDirectoryEntryIterator<'a, D>
where
    D: Device,
{
    pub fn new() -> Self {
        Self {
            call_index: 0,

            peek: Rc::new(|_| None),
            advance: Rc::new(|_| Ok(false)),
            next: Rc::new(|_| None),
        }
    }

    pub fn with_peek<FP>(mut self, peek: FP) -> Self
    where
        FP: Fn(usize) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> + 'a,
    {
        self.peek = Rc::new(peek);
        self
    }

    pub fn with_advance<FA>(mut self, advance: FA) -> Self
    where
        FA: Fn(usize) -> DirectoryEntryIteratorResult<bool, D> + 'a,
    {
        self.advance = Rc::new(advance);
        self
    }

    pub fn with_next<FN>(mut self, next: FN) -> Self
    where
        FN: Fn(usize) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> + 'a,
    {
        self.next = Rc::new(next);
        self
    }
}

impl<D> Debug for ScriptedDirectoryEntryIterator<'_, D>
where
    D: Device,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ScriptedDirectoryEntryIterator")
            .field("call_index", &self.call_index)
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "sync")]
impl<D> ScriptedDirectoryEntryIterator<'_, D>
where
    D: SyncDevice,
{
    pub fn peek(&self) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> {
        (self.peek)(self.call_index)
    }

    pub fn advance(&mut self) -> DirectoryEntryIteratorResult<bool, D> {
        let current_call_index = self.call_index;
        self.call_index += 1;

        (self.advance)(current_call_index)
    }

    pub fn next(&mut self) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> {
        let current_call_index = self.call_index;
        self.call_index += 1;

        (self.next)(current_call_index)
    }
}

#[cfg(feature = "async")]
impl<D> ScriptedDirectoryEntryIterator<'_, D>
where
    D: AsyncDevice,
{
    pub async fn peek_async(&self) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> {
        (self.peek)(self.call_index)
    }

    pub async fn advance_async(&mut self) -> DirectoryEntryIteratorResult<bool, D> {
        let current_call_index = self.call_index;
        self.call_index += 1;

        (self.advance)(current_call_index)
    }

    pub async fn next_async(&mut self) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> {
        let current_call_index = self.call_index;
        self.call_index += 1;

        (self.next)(current_call_index)
    }
}
