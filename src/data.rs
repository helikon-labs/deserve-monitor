use crate::types::{Chain, Endpoint, Provider, ServiceType};

pub const POLKADOT: Chain = Chain {
    id: 0,
    name: "Polkadot",
    genesis_hash: "0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3",
    ss58_prefix: 0,
    relay_chain_id: None,
};

pub const POLKADOT_ASSET_HUB: Chain = Chain {
    id: 1,
    name: "Polkadot Asset Hub",
    genesis_hash: "0x68d56f15f85d3136970ec16946040bc1752654e906147f7e43e9d539d7c3de2f",
    ss58_prefix: 0,
    relay_chain_id: Some(POLKADOT.id),
};

pub const POLKADOT_CORETIME: Chain = Chain {
    id: 2,
    name: "Polkadot Coretime",
    genesis_hash: "0xefb56e30d9b4a24099f88820987d0f45fb645992416535d87650d98e00f46fc4",
    ss58_prefix: 0,
    relay_chain_id: Some(POLKADOT.id),
};

pub const CHAINS: &[Chain] = &[POLKADOT, POLKADOT_ASSET_HUB, POLKADOT_CORETIME];

pub const DESERVE: Provider = Provider {
    id: 0,
    name: "DeServe",
    website: "https://deserve.network",
};

pub const IBP: Provider = Provider {
    id: 1,
    name: "IBP",
    website: "https://ibp.network",
};

pub const DOTTERS: Provider = Provider {
    id: 2,
    name: "Dotters",
    website: "https://dotters.network",
};

pub const DWELLIR: Provider = Provider {
    id: 3,
    name: "Dwellir",
    website: "https://dwellir.com",
};

pub const ON_FINALITY: Provider = Provider {
    id: 4,
    name: "OnFinality",
    website: "",
};

pub const LUCKY_FRIDAY: Provider = Provider {
    id: 5,
    name: "LuckyFriday",
    website: "",
};

pub const PROVIDERS: &[Provider] = &[DESERVE, IBP, DWELLIR, ON_FINALITY, LUCKY_FRIDAY];

pub const ENDPOINTS: &[Endpoint] = &[
    // Polkadot Asset Hub
    Endpoint {
        id: 0,
        chain_id: POLKADOT_ASSET_HUB.id,
        provider_id: DESERVE.id,
        service_type: ServiceType::SubstrateRPC,
        supports_http: true,
        supports_ws: true,
        url: "asset-hub.polkadot.rpc.deserve.network",
    },
    Endpoint {
        id: 1,
        chain_id: POLKADOT_ASSET_HUB.id,
        provider_id: IBP.id,
        service_type: ServiceType::SubstrateRPC,
        supports_http: true,
        supports_ws: true,
        url: "asset-hub-polkadot.ibp.network",
    },
    Endpoint {
        id: 2,
        chain_id: POLKADOT_ASSET_HUB.id,
        provider_id: DOTTERS.id,
        service_type: ServiceType::SubstrateRPC,
        supports_http: true,
        supports_ws: true,
        url: "asset-hub-polkadot.dotters.network",
    },
    // Polkadot Asset Hub ETH RPC
    Endpoint {
        id: 3,
        chain_id: POLKADOT_ASSET_HUB.id,
        provider_id: DESERVE.id,
        service_type: ServiceType::EthereumRPC,
        supports_http: true,
        supports_ws: true,
        url: "asset-hub.polkadot.eth-rpc.deserve.network",
    },
    // Coretime
    Endpoint {
        id: 4,
        chain_id: POLKADOT_CORETIME.id,
        provider_id: DESERVE.id,
        service_type: ServiceType::SubstrateRPC,
        supports_http: true,
        supports_ws: true,
        url: "coretime.polkadot.rpc.deserve.network",
    },
    Endpoint {
        id: 5,
        chain_id: POLKADOT_CORETIME.id,
        provider_id: IBP.id,
        service_type: ServiceType::SubstrateRPC,
        supports_http: true,
        supports_ws: true,
        url: "coretime-polkadot.ibp.network",
    },
    Endpoint {
        id: 6,
        chain_id: POLKADOT_CORETIME.id,
        provider_id: DOTTERS.id,
        service_type: ServiceType::SubstrateRPC,
        supports_http: true,
        supports_ws: true,
        url: "coretime-polkadot.dotters.network",
    },
];
