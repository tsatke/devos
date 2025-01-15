use limine::memory_map::EntryType;
use limine::request::MemoryMapRequest;
use log::debug;

#[used]
#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

pub fn init() {
    if let Some(response) = MEMORY_MAP_REQUEST.get_response() {
        response
            .entries()
            .iter()
            .filter(|e| e.entry_type == EntryType::USABLE)
            .for_each(|entry| {
                debug!("usable memory map entry: 0x{:#X}", entry.base);
            });
    }
    // todo initialize
    debug!("memory initialized")
}
