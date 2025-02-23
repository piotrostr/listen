pub struct Caip2;

impl Caip2 {
    pub const SOLANA: &str = "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp";

    pub const ETHEREUM: &str = "eip155:1";
    pub const BSC: &str = "eip155:56";
    pub const ARBITRUM: &str = "eip155:42161";
    pub const BASE: &str = "eip155:8453";
    pub const BLAST: &str = "eip155:81457";
    pub const AVALANCHE: &str = "eip155:43114";
    pub const POLYGON: &str = "eip155:137";
    pub const SCROLL: &str = "eip155:534352";
    pub const OPTIMISM: &str = "eip155:10";
    pub const LINEA: &str = "eip155:59144";
    pub const GNOSIS: &str = "eip155:100";
    pub const FANTOM: &str = "eip155:250";
    pub const MOONRIVER: &str = "eip155:1285";
    pub const MOONBEAM: &str = "eip155:1284";
    pub const BOBA: &str = "eip155:288";
    pub const MODE: &str = "eip155:34443";
    pub const METIS: &str = "eip155:1088";
    pub const LISK: &str = "eip155:1135";
    pub const AURORA: &str = "eip155:1313161554";
    pub const SEI: &str = "eip155:1329";
    pub const IMMUTABLE: &str = "eip155:13371";
    pub const GRAVITY: &str = "eip155:1625";
    pub const TAIKO: &str = "eip155:167000";
    pub const CRONOS: &str = "eip155:25";
    pub const FRAXTAL: &str = "eip155:252";
    pub const ABSTRACT: &str = "eip155:2741";
    pub const CELO: &str = "eip155:42220";
    pub const WORLD: &str = "eip155:480";
    pub const MANTLE: &str = "eip155:5000";
    pub const BERACHAIN: &str = "eip155:80094";
}

impl Caip2 {
    pub fn from_chain_id(chain_id: u64) -> &'static str {
        match chain_id {
            1 => Caip2::ETHEREUM,
            56 => Caip2::BSC,
            42161 => Caip2::ARBITRUM,
            8453 => Caip2::BASE,
            81457 => Caip2::BLAST,
            43114 => Caip2::AVALANCHE,
            137 => Caip2::POLYGON,
            59144 => Caip2::LINEA,
            100 => Caip2::GNOSIS,
            250 => Caip2::FANTOM,
            1285 => Caip2::MOONRIVER,
            1284 => Caip2::MOONBEAM,
            288 => Caip2::BOBA,
            34443 => Caip2::MODE,
            1088 => Caip2::METIS,
            1135 => Caip2::LISK,
            1313161554 => Caip2::AURORA,
            1329 => Caip2::SEI,
            13371 => Caip2::IMMUTABLE,
            1625 => Caip2::GRAVITY,
            167000 => Caip2::TAIKO,
            25 => Caip2::CRONOS,
            252 => Caip2::FRAXTAL,
            2741 => Caip2::ABSTRACT,
            42220 => Caip2::CELO,
            480 => Caip2::WORLD,
            5000 => Caip2::MANTLE,
            80094 => Caip2::BERACHAIN,
            _ => "eip155:1",
        }
    }
}
