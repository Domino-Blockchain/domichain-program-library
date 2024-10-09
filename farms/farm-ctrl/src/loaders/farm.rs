//! Farms loader.

use {
    crate::{config::Config, loaders::utils::*},
    log::info,
    serde::Deserialize,
    serde_json::{json, Value},
    solana_farm_client::client::FarmClient,
    solana_farm_sdk::{
        farm::{Farm, FarmRoute, FarmType},
        pack::{optional_pubkey_deserialize, pubkey_deserialize, pubkey_slice_deserialize},
        program::protocol::orca::OrcaFarmState,
        refdb::StorageType,
        string::str_to_as64,
        token::GitToken,
    },
    domichain_sdk::{hash::Hasher, pubkey::Pubkey},
    std::str::FromStr,
};

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct JsonRaydiumFarmLegacy {
    name: String,
    lp: String,
    reward: String,
    #[serde(rename = "rewardB", default)]
    reward_b: String,
    #[serde(rename = "isStake")]
    is_stake: bool,
    #[allow(dead_code)]
    fusion: bool,
    legacy: bool,
    dual: bool,
    version: u8,
    #[serde(rename = "programId")]
    program_id: String,
    #[serde(rename = "poolId", deserialize_with = "pubkey_deserialize")]
    farm_id: Pubkey,
    #[serde(rename = "poolAuthority", deserialize_with = "pubkey_deserialize")]
    farm_authority: Pubkey,
    #[serde(rename = "poolLpTokenAccount", deserialize_with = "pubkey_deserialize")]
    farm_lp_token_account: Pubkey,
    #[serde(
        rename = "poolRewardTokenAccount",
        deserialize_with = "pubkey_deserialize"
    )]
    farm_reward_token_account: Pubkey,
    #[serde(
        rename = "poolRewardTokenAccountB",
        deserialize_with = "optional_pubkey_deserialize",
        default
    )]
    farm_reward_token_account_b: Option<Pubkey>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct JsonRaydiumFarm {
    #[serde(deserialize_with = "pubkey_deserialize")]
    id: Pubkey,
    #[serde(rename = "lpMint", deserialize_with = "pubkey_deserialize")]
    lp_mint: Pubkey,
    #[serde(rename = "rewardMints", deserialize_with = "pubkey_slice_deserialize")]
    reward_mints: Vec<Pubkey>,
    version: u8,
    #[serde(rename = "programId", deserialize_with = "pubkey_deserialize")]
    program_id: Pubkey,
    #[serde(deserialize_with = "pubkey_deserialize")]
    authority: Pubkey,
    #[serde(rename = "lpVault", deserialize_with = "pubkey_deserialize")]
    lp_vault: Pubkey,
    #[serde(rename = "rewardVaults", deserialize_with = "pubkey_slice_deserialize")]
    reward_vaults: Vec<Pubkey>,
    upcoming: bool,
}

#[derive(Deserialize, Debug)]
struct JsonSaberFarm {
    #[allow(dead_code)]
    name: String,
    tokens: Vec<GitToken>,
    #[serde(rename = "lpToken")]
    lp_token: GitToken,
    #[serde(deserialize_with = "pubkey_deserialize")]
    quarry: Pubkey,
}

#[derive(Deserialize, Debug)]
pub struct JsonOrcaFarm {
    pub name: String,
    #[serde(deserialize_with = "pubkey_deserialize")]
    pub address: Pubkey,
    #[serde(rename = "farmTokenMint", deserialize_with = "pubkey_deserialize")]
    pub farm_token_mint: Pubkey,
    #[serde(rename = "rewardTokenMint", deserialize_with = "pubkey_deserialize")]
    pub reward_token_mint: Pubkey,
    #[serde(rename = "rewardTokenDecimals")]
    pub reward_token_decimals: u8,
    #[serde(rename = "baseTokenMint", deserialize_with = "pubkey_deserialize")]
    pub base_token_mint: Pubkey,
    #[serde(rename = "baseTokenDecimals")]
    pub base_token_decimals: u8,
}

pub fn load(client: &FarmClient, config: &Config, data: &str, remove_mode: bool) {
    let parsed: Value = serde_json::from_str(data).unwrap();
    let last_index = client
        .get_refdb_last_index(&StorageType::Farm.to_string())
        .expect("Farm RefDB query error");

    if parsed["name"] == "Raydium Farms" {
        load_raydium_farm_legacy(client, config, remove_mode, &parsed, last_index);
    } else if parsed["name"] == "Raydium Mainnet Farm Pools" {
        load_raydium_farm(client, config, remove_mode, &parsed, last_index);
    } else if parsed["name"] == "Orca Farms" {
        load_orca_farm(client, config, remove_mode, &parsed, last_index);
    } else if parsed["pools"] != json!(null) && parsed["addresses"] != json!(null) {
        load_saber_farm(client, config, remove_mode, &parsed, last_index);
    } else {
        panic!("Unsupported farms file");
    }
}

fn load_raydium_farm_legacy(
    client: &FarmClient,
    config: &Config,
    remove_mode: bool,
    parsed: &Value,
    last_index: u32,
) {
    let mut last_index = last_index;
    let router_id = client.get_program_id("RaydiumRouter").unwrap();
    let farms = parsed["farms"].as_array().unwrap();
    for val in farms {
        let json_farm: JsonRaydiumFarmLegacy = serde_json::from_value(val.clone()).unwrap();
        let lp_token = client.get_token(&json_farm.lp.to_uppercase()).unwrap();
        let (pool_name, _) = if FarmClient::is_liquidity_token(&lp_token.name) {
            FarmClient::extract_pool_name_and_version(&lp_token.name).unwrap()
        } else {
            ("RDM.".to_string() + &lp_token.name, 0)
        };
        let name = format!("{}-V{}", pool_name, json_farm.version);
        if !remove_mode {
            if json_farm.legacy {
                info!("Skipping legacy Farm \"{}\"...", name);
                continue;
            }
            if config.skip_existing && client.get_farm(&name).is_ok() {
                info!("Skipping existing Farm \"{}\"...", name);
                continue;
            }
            info!("Writing Farm \"{}\" to on-chain RefDB...", name);
        } else {
            info!("Removing Farm \"{}\" from on-chain RefDB...", name);
            client.remove_farm(config.keypair.as_ref(), &name).unwrap();
            continue;
        }
        let (index, counter) = if let Ok(farm) = client.get_farm(&name) {
            (farm.refdb_index, farm.refdb_counter)
        } else {
            last_index += 1;
            (Some(last_index - 1), 0u16)
        };
        let farm = Farm {
            name: str_to_as64(&name).unwrap(),
            version: json_farm.version as u16,
            farm_type: if json_farm.dual {
                FarmType::DualReward
            } else if json_farm.is_stake {
                FarmType::ProtocolTokenStake
            } else {
                FarmType::SingleReward
            },
            official: true,
            refdb_index: index,
            refdb_counter: counter,
            lp_token_ref: Some(client.get_token_ref(&json_farm.lp.to_uppercase()).unwrap()),
            first_reward_token_ref: Some(
                client
                    .get_token_ref(&json_farm.reward.to_uppercase())
                    .unwrap(),
            ),
            second_reward_token_ref: if json_farm.reward_b.is_empty() {
                None
            } else {
                Some(
                    client
                        .get_token_ref(&json_farm.reward_b.to_uppercase())
                        .unwrap(),
                )
            },
            router_program_id: router_id,
            farm_program_id: convert_raydium_program_id(client, &json_farm.program_id),
            route: FarmRoute::Raydium {
                farm_id: json_farm.farm_id,
                farm_authority: json_farm.farm_authority,
                farm_lp_token_account: json_farm.farm_lp_token_account,
                farm_first_reward_token_account: json_farm.farm_reward_token_account,
                farm_second_reward_token_account: json_farm.farm_reward_token_account_b,
            },
        };

        client.add_farm(config.keypair.as_ref(), farm).unwrap();
    }
}

fn load_raydium_farm(
    client: &FarmClient,
    config: &Config,
    remove_mode: bool,
    parsed: &Value,
    last_index: u32,
) {
    let mut last_index = last_index;
    let router_id = client.get_program_id("RaydiumRouter").unwrap();
    let farms = parsed["official"].as_array().unwrap();
    for val in farms {
        let json_farm: JsonRaydiumFarm = serde_json::from_value(val.clone()).unwrap();
        let lp_token = if let Ok(token) = client.get_token_with_mint(&json_farm.lp_mint) {
            token
        } else {
            info!(
                "Skipping Farm with unrecognized lp token {}",
                json_farm.lp_mint
            );
            continue;
        };
        let (pool_name, _) = if FarmClient::is_liquidity_token(&lp_token.name) {
            FarmClient::extract_pool_name_and_version(&lp_token.name).unwrap()
        } else {
            ("RDM.".to_string() + &lp_token.name, 0)
        };
        let name = format!("{}-V{}", pool_name, json_farm.version);
        if !remove_mode {
            if config.skip_existing && client.get_farm(&name).is_ok() {
                info!("Skipping existing Farm \"{}\"...", name);
                continue;
            }
            info!("Writing Farm \"{}\" to on-chain RefDB...", name);
        } else {
            info!("Removing Farm \"{}\" from on-chain RefDB...", name);
            client.remove_farm(config.keypair.as_ref(), &name).unwrap();
            continue;
        }
        let (index, counter) = if let Ok(farm) = client.get_farm(&name) {
            (farm.refdb_index, farm.refdb_counter)
        } else {
            last_index += 1;
            (Some(last_index - 1), 0u16)
        };
        let farm = Farm {
            name: str_to_as64(&name).unwrap(),
            version: json_farm.version as u16,
            farm_type: if json_farm.reward_mints.len() > 1 {
                FarmType::DualReward
            } else if json_farm.lp_mint
                == Pubkey::from_str("4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R").unwrap()
            {
                FarmType::ProtocolTokenStake
            } else {
                FarmType::SingleReward
            },
            official: true,
            refdb_index: index,
            refdb_counter: counter,
            lp_token_ref: Some(client.get_token_ref(&lp_token.name).unwrap()),
            first_reward_token_ref: Some(get_token_ref_with_mint(
                client,
                &json_farm.reward_mints[0],
            )),
            second_reward_token_ref: if json_farm.reward_mints.len() < 2 {
                None
            } else {
                Some(get_token_ref_with_mint(client, &json_farm.reward_mints[1]))
            },
            router_program_id: router_id,
            farm_program_id: json_farm.program_id,
            route: FarmRoute::Raydium {
                farm_id: json_farm.id,
                farm_authority: json_farm.authority,
                farm_lp_token_account: json_farm.lp_vault,
                farm_first_reward_token_account: json_farm.reward_vaults[0],
                farm_second_reward_token_account: if json_farm.reward_vaults.len() < 2 {
                    None
                } else {
                    Some(json_farm.reward_vaults[1])
                },
            },
        };

        client.add_farm(config.keypair.as_ref(), farm).unwrap();
    }
}

fn load_saber_farm(
    client: &FarmClient,
    config: &Config,
    remove_mode: bool,
    parsed: &Value,
    last_index: u32,
) {
    let mut last_index = last_index;
    let pools = parsed["pools"].as_array().unwrap();
    let router_id = client.get_program_id("SaberRouter").unwrap();

    let farm_program_id = client.get_program_id("SaberQuarryMine").unwrap();
    let redeemer_program = client.get_program_id("SaberRedeemer").unwrap();
    let mint_proxy_program = client.get_program_id("SaberMintProxy").unwrap();
    let redeemer = json_to_pubkey(&parsed["addresses"]["redeemer"]);
    let sbr_mint = client.get_token("SBR").unwrap().mint;
    let sbr_vault =
        spl_associated_token_account::get_associated_token_address(&redeemer, &sbr_mint);
    let rewarder = json_to_pubkey(&parsed["addresses"]["rewarder"]);
    let iou_mint = client.get_token("IOU").unwrap().mint;
    let iou_fees_account =
        spl_associated_token_account::get_associated_token_address(&rewarder, &iou_mint);
    let mint_wrapper = json_to_pubkey(&parsed["addresses"]["mintWrapper"]);
    let mint_wrapper_program = client.get_program_id("SaberMintWrapper").unwrap();

    // minter
    let minter = Pubkey::find_program_address(
        &[
            b"MintWrapperMinter",
            &mint_wrapper.to_bytes(),
            &rewarder.to_bytes(),
        ],
        &mint_wrapper_program,
    )
    .0;

    // mint_proxy_authority
    let registry_signer = Pubkey::find_program_address(&[], &mint_proxy_program).0;
    let mut buffer = vec![];
    buffer.extend_from_slice(&registry_signer.to_bytes());
    buffer.extend_from_slice(b"unversioned");
    buffer.extend_from_slice(&mint_proxy_program.to_bytes());
    let mut hasher = Hasher::default();
    hasher.hash(buffer.as_slice());
    let mint_proxy_authority = Pubkey::new(hasher.result().as_ref());

    // mint_proxy_state
    let mint_proxy_state = Pubkey::find_program_address(
        &[b"SaberMintProxy", &mint_proxy_authority.to_bytes()],
        &mint_proxy_program,
    )
    .0;

    // minter info
    let minter_info =
        Pubkey::find_program_address(&[b"anchor", &redeemer.to_bytes()], &mint_proxy_program).0;

    for val in pools {
        let json_farm: JsonSaberFarm = serde_json::from_value(val.clone()).unwrap();
        let name = get_saber_pool_name(client, &json_farm.tokens[0], &json_farm.tokens[1]);
        if !remove_mode {
            if config.skip_existing && client.get_farm(&name).is_ok() {
                info!("Skipping existing Farm \"{}\"...", name);
                continue;
            }
            info!("Writing Farm \"{}\" to on-chain RefDB...", name);
        } else {
            info!("Removing Farm \"{}\" from on-chain RefDB...", name);
            client.remove_farm(config.keypair.as_ref(), &name).unwrap();
            continue;
        }
        let (index, counter) = if let Ok(farm) = client.get_farm(&name) {
            (farm.refdb_index, farm.refdb_counter)
        } else {
            last_index += 1;
            (Some(last_index - 1), 0u16)
        };
        let farm_token_name = get_saber_token_name(client, &json_farm.lp_token);
        if json_farm.tokens[0].address
            != convert_pubkey(val["swap"]["state"]["tokenA"]["mint"].as_str().unwrap())
            || json_farm.tokens[1].address
                != convert_pubkey(val["swap"]["state"]["tokenB"]["mint"].as_str().unwrap())
        {
            panic!("Farm metadata mismatch");
        }
        let farm = Farm {
            name: str_to_as64(&name).unwrap(),
            version: 1u16,
            farm_type: FarmType::SingleReward,
            official: true,
            refdb_index: index,
            refdb_counter: counter,
            lp_token_ref: Some(client.get_token_ref(&farm_token_name).unwrap()),
            first_reward_token_ref: Some(client.get_token_ref("SBR").unwrap()),
            second_reward_token_ref: Some(client.get_token_ref("IOU").unwrap()),
            router_program_id: router_id,
            farm_program_id,
            route: FarmRoute::Saber {
                quarry: json_farm.quarry,
                rewarder,
                redeemer,
                redeemer_program,
                minter,
                mint_wrapper,
                mint_wrapper_program,
                iou_fees_account,
                sbr_vault,
                mint_proxy_program,
                mint_proxy_authority,
                mint_proxy_state,
                minter_info,
            },
        };

        client.add_farm(config.keypair.as_ref(), farm).unwrap();
    }
}

fn load_orca_farm(
    client: &FarmClient,
    config: &Config,
    remove_mode: bool,
    parsed: &Value,
    last_index: u32,
) {
    let mut last_index = last_index;
    let router_id = client.get_program_id("OrcaRouter").unwrap();
    let farm_program_id = client.get_program_id("OrcaStake").unwrap();
    let farms = parsed["farms"].as_array().unwrap();
    for val in farms {
        let json_farm: JsonOrcaFarm = serde_json::from_value(val.clone()).unwrap();
        let lp_token = client
            .get_token_with_mint(&json_farm.base_token_mint)
            .unwrap();
        let (pool_name, _) = if FarmClient::is_liquidity_token(&lp_token.name) {
            FarmClient::extract_pool_name_and_version(&lp_token.name).unwrap()
        } else {
            ("ORC.".to_string() + &lp_token.name, 0)
        };
        let name = if pool_name.ends_with("-AQ") {
            format!("{}-DD-V1", &pool_name[..pool_name.len() - 3])
        } else {
            format!("{}-AQ-V1", pool_name)
        };
        if !remove_mode {
            if config.skip_existing && client.get_farm(&name).is_ok() {
                info!("Skipping existing Farm \"{}\"...", name);
                continue;
            }
            info!("Writing Farm \"{}\" to on-chain RefDB...", name);
        } else {
            info!("Removing Farm \"{}\" from on-chain RefDB...", name);
            client.remove_farm(config.keypair.as_ref(), &name).unwrap();
            continue;
        }
        let (index, counter) = if let Ok(farm) = client.get_farm(&name) {
            (farm.refdb_index, farm.refdb_counter)
        } else {
            last_index += 1;
            (Some(last_index - 1), 0u16)
        };
        let farm_data = client
            .rpc_client
            .get_account_data(&json_farm.address)
            .unwrap();
        let farm_state = OrcaFarmState::unpack(&farm_data).unwrap();
        let farm = Farm {
            name: str_to_as64(&name).unwrap(),
            version: 1,
            farm_type: FarmType::SingleReward,
            official: true,
            refdb_index: index,
            refdb_counter: counter,
            lp_token_ref: Some(client.get_token_ref(&lp_token.name).unwrap()),
            first_reward_token_ref: Some(get_token_ref_with_mint(
                client,
                &json_farm.reward_token_mint,
            )),
            second_reward_token_ref: None,
            router_program_id: router_id,
            farm_program_id,
            route: FarmRoute::Orca {
                farm_id: json_farm.address,
                farm_authority: Pubkey::find_program_address(
                    &[&json_farm.address.to_bytes()],
                    &farm_program_id,
                )
                .0,
                farm_token_ref: get_token_ref_with_mint(client, &json_farm.farm_token_mint),
                base_token_vault: farm_state.base_token_vault,
                reward_token_vault: farm_state.reward_token_vault,
            },
        };

        client.add_farm(config.keypair.as_ref(), farm).unwrap();
    }
}
