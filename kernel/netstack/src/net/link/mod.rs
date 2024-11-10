pub mod ethernet;
pub mod interface;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DataLinkProtocol {
    Ethernet,
}
