use zksync_ethers_rs::types::H160;

pub const DEFAULT_L1_RPC_URL: &str = "http://localhost:8545";
pub const DEFAULT_L2_RPC_URL: &str = "http://localhost:3050";
pub const DEFAULT_L2_EXPLORER_URL: &str = "http://localhost:3010";
pub const DEFAULT_L1_EXPLORER_URL: &str = "";
pub const DEFAULT_PRIVATE_KEY: &str =
    "0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8";
// 0xa13c10c0d5bd6f79041b9835c63f91de35a15883
pub const DEFAULT_ADDRESS: H160 = H160([
    0xa1, 0x3c, 0x10, 0xc0, 0xd5, 0xbd, 0x6f, 0x79, 0x04, 0x1b, 0x98, 0x35, 0xc6, 0x3f, 0x91, 0xde,
    0x35, 0xa1, 0x58, 0x83,
]);
// 0x5E6D086F5eC079ADFF4FB3774CDf3e8D6a34F7E9
pub const DEFAULT_CONTRACT_ADDRESS: H160 = H160([
    0x5E, 0x6D, 0x08, 0x6F, 0x5e, 0xC0, 0x79, 0xAD, 0xFF, 0x4F, 0xB3, 0x77, 0x4C, 0xdf, 0x3e, 0x8D,
    0x6a, 0x34, 0xF7, 0xE9,
]);