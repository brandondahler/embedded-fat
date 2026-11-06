use crate::allocation_table::AllocationTable;

#[derive(Debug)]
pub struct Device<S> {
    pub stream: S,

    pub allocation_table: AllocationTable,
}
