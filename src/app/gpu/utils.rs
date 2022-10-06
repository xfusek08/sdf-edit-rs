
pub trait BufferExt {
    fn update_buffer_sync(&self, contents: &[u8]);
}

impl BufferExt for wgpu::Buffer {
    #[profiler::function]
    fn update_buffer_sync(&self, contents: &[u8]) {
        let contents_size = contents.len();
        
        profiler::call!(
            self.slice(..)
            .get_mapped_range_mut()[..contents_size]
            .copy_from_slice(contents)
        );
        profiler::call!(self.unmap());
    }
}
