use crate::directory_entry::{
    DirectoryEntry, DirectoryEntryIterationError, DirectoryEntryIterator,
};
use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::RefCell;
use core::fmt::Debug;
use embedded_io::{ErrorType, SeekFrom};

#[cfg(feature = "sync")]
use embedded_io::{Read, Seek};

use crate::mock::{CoreError, IoError};
#[cfg(feature = "async")]
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek};

pub struct ScriptedDirectoryEntryIterator<'a, S>
where
    S: ErrorType,
{
    call_index: usize,

    peek: Rc<
        dyn Fn(usize) -> Result<Option<DirectoryEntry>, DirectoryEntryIterationError<S::Error>>
            + 'a,
    >,
    advance: Rc<dyn Fn(usize) -> Result<bool, DirectoryEntryIterationError<S::Error>> + 'a>,
    next: Rc<
        dyn Fn(usize) -> Result<Option<DirectoryEntry>, DirectoryEntryIterationError<S::Error>>
            + 'a,
    >,
}

impl<'a, S> ScriptedDirectoryEntryIterator<'a, S>
where
    S: ErrorType,
{
    pub fn new() -> Self {
        Self {
            call_index: 0,

            peek: Rc::new(|_| Ok(None)),
            advance: Rc::new(|_| Ok(false)),
            next: Rc::new(|_| Ok(None)),
        }
    }

    pub fn with_peek<FP>(mut self, peek: FP) -> Self
    where
        FP: Fn(usize) -> Result<Option<DirectoryEntry>, DirectoryEntryIterationError<S::Error>>
            + 'a,
    {
        self.peek = Rc::new(peek);
        self
    }

    pub fn with_advance<FA>(mut self, advance: FA) -> Self
    where
        FA: Fn(usize) -> Result<bool, DirectoryEntryIterationError<S::Error>> + 'a,
    {
        self.advance = Rc::new(advance);
        self
    }

    pub fn with_next<FN>(mut self, next: FN) -> Self
    where
        FN: Fn(usize) -> Result<Option<DirectoryEntry>, DirectoryEntryIterationError<S::Error>>
            + 'a,
    {
        self.next = Rc::new(next);
        self
    }
}

impl<S> Clone for ScriptedDirectoryEntryIterator<'_, S>
where
    S: ErrorType,
{
    fn clone(&self) -> Self {
        Self {
            call_index: self.call_index,

            advance: self.advance.clone(),
            next: self.next.clone(),
            peek: self.peek.clone(),
        }
    }
}

impl<S> Debug for ScriptedDirectoryEntryIterator<'_, S>
where
    S: ErrorType,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ScriptedDirectoryEntryIterator")
            .field("call_index", &self.call_index)
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "sync")]
impl<S> ScriptedDirectoryEntryIterator<'_, S>
where
    S: ErrorType,
{
    pub fn peek(&self) -> Result<Option<DirectoryEntry>, DirectoryEntryIterationError<S::Error>> {
        (self.peek)(self.call_index)
    }

    pub fn advance(&mut self) -> Result<bool, DirectoryEntryIterationError<S::Error>> {
        let current_call_index = self.call_index;
        self.call_index += 1;

        (self.advance)(current_call_index)
    }

    pub fn next(
        &mut self,
    ) -> Result<Option<DirectoryEntry>, DirectoryEntryIterationError<S::Error>> {
        let current_call_index = self.call_index;
        self.call_index += 1;

        (self.next)(current_call_index)
    }
}

#[cfg(feature = "async")]
impl<S> ScriptedDirectoryEntryIterator<'_, S>
where
    S: ErrorType,
{
    pub async fn peek_async(
        &self,
    ) -> Result<Option<DirectoryEntry>, DirectoryEntryIterationError<S::Error>> {
        (self.peek)(self.call_index)
    }

    pub async fn advance_async(&mut self) -> Result<bool, DirectoryEntryIterationError<S::Error>> {
        let current_call_index = self.call_index;
        self.call_index += 1;

        (self.advance)(current_call_index)
    }

    pub async fn next_async(
        &mut self,
    ) -> Result<Option<DirectoryEntry>, DirectoryEntryIterationError<S::Error>> {
        let current_call_index = self.call_index;
        self.call_index += 1;

        (self.next)(current_call_index)
    }
}
