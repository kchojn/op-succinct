use alloy_primitives::{Address, B256, FixedBytes};
use anyhow::{Context, Result};
use reqwest::Url;
use serde::Serialize;
use sp1_sdk::SP1VerifyingKey;
use op_succinct_client_utils::types::AggregationOutputs;

#[derive(Serialize)]
struct SubmitReq {
    superblock_number: u64,
    superblock_hash: B256,
    chain_id: u32,
    prover_address: Address,
    l1_head: B256,
    aggregation_outputs: AggregationOutputs,
    l2_start_block: u64,
    agg_vk: SP1VerifyingKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    proof: Option<Vec<u8>>,
}

pub fn build_aggregation_outputs(
    l1_head: B256,
    l2_pre_root: B256,
    l2_post_root: B256,
    l2_block_number: u64,
    rollup_config_hash: B256,
    multi_block_vkey: B256,
    prover_address: Address,
) -> AggregationOutputs {
    AggregationOutputs {
        l1Head: l1_head,
        l2PreRoot: l2_pre_root,
        l2PostRoot: l2_post_root,
        l2BlockNumber: l2_block_number,
        rollupConfigHash: rollup_config_hash,
        multiBlockVKey: multi_block_vkey,
        proverAddress: prover_address,
    }
}

/// Post a submission to the shared publisher
pub async fn submit_to_publisher(
    endpoint: &Url,
    superblock_number: u64,
    superblock_hash: B256,
    chain_id_l2: u32,
    prover_address: Address,
    l1_head: B256,
    aggregation_outputs: AggregationOutputs,
    l2_start_block: u64,
    agg_vk: &SP1VerifyingKey,
    proof_bytes: Option<&[u8]>,
) -> Result<()> {
    let client = reqwest::Client::new();

    let body = SubmitReq {
        superblock_number,
        superblock_hash,
        chain_id: chain_id_l2,
        prover_address,
        l1_head,
        aggregation_outputs,
        l2_start_block,
        agg_vk: agg_vk.clone(),
        proof: proof_bytes.map(|p| p.to_vec()),
    };

    let resp = client
        .post(endpoint.clone())
        .json(&body)
        .send()
        .await
        .context("failed to submit to shared publisher")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        anyhow::bail!("publisher responded with {}: {}", status, text);
    }

    Ok(())
}
