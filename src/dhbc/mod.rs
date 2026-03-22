pub mod detector;
pub mod append_dhcp;

pub use detector::get_dhbc_info;
pub use append_dhcp::append_dhcp_dns;