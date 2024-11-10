pub mod ethernet;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DataLinkProtocol {
    Ethernet,
}
