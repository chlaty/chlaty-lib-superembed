

pub const SOURCE_HOST: &str = "simkl.com";
pub const SOURCE_REFERER: &str = "https://simkl.com/";
pub const SOURCE_ORIGIN: &str = "https://simkl.com";

pub const SERVER_ORIGIN: &str = "https://simkl.com";
pub const SERVER_REFERER: &str = "https://simkl.com/";
pub const SERVER_HOST: &str = "simkl.com";

pub const DUMMY_VERIFY_TOKENS: [&str; 5] = [
    "ZEhKMVpTLVF0LVBTLVF0LVB6QS1RTFMtUXpPLTBJdEwtMC1WM05qSTROamMtUU9Ea3otUC0wYy01",
    "ZEhKMVpTLVF0LVBTLVF0TkRrMkxTLVF6Ty0wLVB0TC0wLVYzTmpJNE5qWTUtUERBMy1QalUtNQ==",
    "ZEhKMVpTLVF0LVBTLVF0Ti0wSXdMUy1Rek56LVZ0TC0wLVYzTmpJNE5qSTMtUHprMS1QRGstNQ==",
    "ZEhKMVpTLVF0LVBTLVF0TnpVeUxTLVF6Ty0wLVZ0TC0wLVYzTmpJNE56LVAtUS1QREkzTnpJLTU",
    "ZEhKMVpTLVF0LVBTLVF0TnpZeExTLVF6T0RndEwtMC1WM05qSTROei1QMU56LVAyLVAtMEktNQ==",
];


pub mod search;
pub mod get_episode_list;
pub mod get_episode_server;
pub mod get_server;
pub mod free_ptr;

#[cfg(test)]
mod test;