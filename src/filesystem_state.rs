use crate::allocation_table::AllocationTable;
use embedded_io::ErrorType;

#[derive(Debug)]
pub struct FilesystemState<S>
where
    S: ErrorType,
{
    pub stream: S,

    pub free_cluster_scan_start_number: u32,
}

impl<S> FilesystemState<S>
where
    S: ErrorType,
{
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            free_cluster_scan_start_number: 2,
        }
    }
}
