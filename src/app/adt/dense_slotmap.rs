
#[derive(Clone,Debug)]
pub struct DenseSlotMap<T> {
    entries: Vec<T>,
    metadata: Vec<MetaEntry>,
    free_meta_index: usize,
    len: usize,
}

#[derive(Debug, Clone)]
pub struct VersionedIndex {
    meta_index: usize,
    version: u32,
}

#[derive(Debug, Clone)]
pub struct MetaEntry {
    index: usize,
    version: u32,
    empty: bool,
}

impl MetaEntry {
    pub fn new(index: usize) -> Self {
        Self { index, version: 0, empty: false }
    }
}

// public impl
impl<T> DenseSlotMap<T> {
    /// inserts entry to first free position
    pub fn add(&mut self, entry: T) -> VersionedIndex {
        // vectors all always densely packed new entry goes to back
        self.entries.push(entry);
        
        let entry_index = self.entries.len();
        let meta_index = self.free_meta_index;
        
        // update meta record
        
        // when meta there is not existing record on meta_index -> push new
        if meta_index >= self.metadata.len() {
            self.metadata.push(MetaEntry::new(meta_index));
        }
        
        // update meta entry in self.metadata:
        let mut meta_entry = &mut self.metadata[meta_index];
        meta_entry.index = entry_index;
        meta_entry.version += 1;
        meta_entry.empty = false;
        
        // set next free meta index
        self.free_meta_index = self.find_free_meta_index_from(meta_index);
        
        VersionedIndex {
            meta_index,
            version: meta_entry.version
        }
    }
    
    pub fn as_vec(&self) -> &Vec<T> {
        &self.entries
    }
}

// private impl
impl<T> DenseSlotMap<T> {
    fn find_free_meta_index_from(&self, from: usize) -> usize {
        let mut current_index = from;
        while current_index < self.metadata.len() && !self.metadata[current_index].empty {
            current_index += 1;
        }
        current_index
    }
}
