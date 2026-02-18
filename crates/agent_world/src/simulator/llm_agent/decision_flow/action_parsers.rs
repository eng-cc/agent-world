use super::super::super::social::{SocialAdjudicationDecision, SocialStake};
use super::super::super::types::{Action, PowerOrderSide, ResourceOwner, PPM_BASE};
use super::{parse_owner_spec, parse_resource_kind, LlmDecisionPayload, LlmSocialStakePayload};
use std::collections::BTreeMap;

pub(super) fn parse_market_or_social_action(
    decision: &str,
    parsed: &LlmDecisionPayload,
    agent_id: &str,
) -> Option<Result<Action, String>> {
    match decision {
        "buy_power" => Some(parse_buy_power(parsed, agent_id)),
        "sell_power" => Some(parse_sell_power(parsed, agent_id)),
        "place_power_order" => Some(parse_place_power_order(parsed, agent_id)),
        "cancel_power_order" => Some(parse_cancel_power_order(parsed, agent_id)),
        "compile_module_artifact_from_source" => {
            Some(parse_compile_module_artifact_from_source(parsed, agent_id))
        }
        "deploy_module_artifact" => Some(parse_deploy_module_artifact(parsed, agent_id)),
        "install_module_from_artifact" => {
            Some(parse_install_module_from_artifact(parsed, agent_id))
        }
        "publish_social_fact" => Some(parse_publish_social_fact(parsed, agent_id)),
        "challenge_social_fact" => Some(parse_challenge_social_fact(parsed, agent_id)),
        "adjudicate_social_fact" => Some(parse_adjudicate_social_fact(parsed, agent_id)),
        "revoke_social_fact" => Some(parse_revoke_social_fact(parsed, agent_id)),
        "declare_social_edge" => Some(parse_declare_social_edge(parsed, agent_id)),
        _ => None,
    }
}

fn parse_buy_power(parsed: &LlmDecisionPayload, agent_id: &str) -> Result<Action, String> {
    let buyer = parse_required_owner(parsed.buyer.as_deref(), "buy_power", "buyer", agent_id)?;
    let seller = parse_required_owner(parsed.seller.as_deref(), "buy_power", "seller", agent_id)?;
    let amount = parse_positive_i64(parsed.amount, "buy_power", "amount")?;
    let price_per_pu =
        parse_non_negative_i64_with_default(parsed.price_per_pu, "buy_power", "price_per_pu", 0)?;
    Ok(Action::BuyPower {
        buyer,
        seller,
        amount,
        price_per_pu,
    })
}

fn parse_sell_power(parsed: &LlmDecisionPayload, agent_id: &str) -> Result<Action, String> {
    let seller = parse_required_owner(parsed.seller.as_deref(), "sell_power", "seller", agent_id)?;
    let buyer = parse_required_owner(parsed.buyer.as_deref(), "sell_power", "buyer", agent_id)?;
    let amount = parse_positive_i64(parsed.amount, "sell_power", "amount")?;
    let price_per_pu =
        parse_non_negative_i64_with_default(parsed.price_per_pu, "sell_power", "price_per_pu", 0)?;
    Ok(Action::SellPower {
        seller,
        buyer,
        amount,
        price_per_pu,
    })
}

fn parse_place_power_order(parsed: &LlmDecisionPayload, agent_id: &str) -> Result<Action, String> {
    let owner = parse_owner_or_self(parsed.owner.as_deref(), agent_id)?;
    let side_raw = parse_required_text(parsed.side.as_deref(), "place_power_order", "side")?;
    let side = parse_power_order_side(side_raw.as_str())
        .ok_or_else(|| format!("place_power_order invalid side: {side_raw}"))?;
    let amount = parse_positive_i64(parsed.amount, "place_power_order", "amount")?;
    let limit_price_per_pu = parse_non_negative_i64_with_default(
        parsed.limit_price_per_pu,
        "place_power_order",
        "limit_price_per_pu",
        0,
    )?;
    Ok(Action::PlacePowerOrder {
        owner,
        side,
        amount,
        limit_price_per_pu,
    })
}

fn parse_cancel_power_order(parsed: &LlmDecisionPayload, agent_id: &str) -> Result<Action, String> {
    let owner = parse_owner_or_self(parsed.owner.as_deref(), agent_id)?;
    let order_id = parse_positive_u64(parsed.order_id, "cancel_power_order", "order_id")?;
    Ok(Action::CancelPowerOrder { owner, order_id })
}

fn parse_compile_module_artifact_from_source(
    parsed: &LlmDecisionPayload,
    agent_id: &str,
) -> Result<Action, String> {
    let publisher_agent_id = parse_agent_identity_or_self(
        parsed.publisher.as_deref(),
        "compile_module_artifact_from_source",
        "publisher",
        agent_id,
    )?;
    let module_id = parse_required_text(
        parsed.module_id.as_deref(),
        "compile_module_artifact_from_source",
        "module_id",
    )?;
    let manifest_path = parse_required_text(
        parsed.manifest_path.as_deref(),
        "compile_module_artifact_from_source",
        "manifest_path",
    )?;
    let source_files = parse_source_files(
        parsed.source_files.as_ref(),
        "compile_module_artifact_from_source",
        "source_files",
    )?;
    Ok(Action::CompileModuleArtifactFromSource {
        publisher_agent_id,
        module_id,
        manifest_path,
        source_files,
    })
}

fn parse_deploy_module_artifact(
    parsed: &LlmDecisionPayload,
    agent_id: &str,
) -> Result<Action, String> {
    let publisher_agent_id = parse_agent_identity_or_self(
        parsed.publisher.as_deref(),
        "deploy_module_artifact",
        "publisher",
        agent_id,
    )?;
    let wasm_hash = parse_required_text(
        parsed.wasm_hash.as_deref(),
        "deploy_module_artifact",
        "wasm_hash",
    )?;
    let wasm_bytes = parse_hex_bytes(
        parsed.wasm_bytes_hex.as_deref(),
        "deploy_module_artifact",
        "wasm_bytes_hex",
    )?;
    let module_id_hint = parsed
        .module_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    Ok(Action::DeployModuleArtifact {
        publisher_agent_id,
        wasm_hash,
        wasm_bytes,
        module_id_hint,
    })
}

fn parse_install_module_from_artifact(
    parsed: &LlmDecisionPayload,
    agent_id: &str,
) -> Result<Action, String> {
    let installer_agent_id = parse_agent_identity_or_self(
        parsed.installer.as_deref(),
        "install_module_from_artifact",
        "installer",
        agent_id,
    )?;
    let module_id = parse_required_text(
        parsed.module_id.as_deref(),
        "install_module_from_artifact",
        "module_id",
    )?;
    let module_version = parsed
        .module_version
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("0.1.0")
        .to_string();
    let wasm_hash = parse_required_text(
        parsed.wasm_hash.as_deref(),
        "install_module_from_artifact",
        "wasm_hash",
    )?;
    let activate = parsed.activate.unwrap_or(true);
    Ok(Action::InstallModuleFromArtifact {
        installer_agent_id,
        module_id,
        module_version,
        wasm_hash,
        activate,
    })
}

fn parse_publish_social_fact(
    parsed: &LlmDecisionPayload,
    agent_id: &str,
) -> Result<Action, String> {
    let actor = parse_owner_or_self(parsed.actor.as_deref(), agent_id)?;
    let schema_id = parse_required_text(
        parsed.schema_id.as_deref(),
        "publish_social_fact",
        "schema_id",
    )?;
    let subject = parse_required_owner(
        parsed.subject.as_deref(),
        "publish_social_fact",
        "subject",
        agent_id,
    )?;
    let object = match parsed.object.as_deref() {
        Some(owner) => Some(parse_owner_spec(owner, agent_id)?),
        None => None,
    };
    let claim = parse_required_text(parsed.claim.as_deref(), "publish_social_fact", "claim")?;
    let confidence_ppm = parsed.confidence_ppm.unwrap_or(PPM_BASE);
    if !(1..=PPM_BASE).contains(&confidence_ppm) {
        return Err(format!(
            "publish_social_fact confidence_ppm out of range: {confidence_ppm} (expected 1..={PPM_BASE})"
        ));
    }
    let evidence_event_ids = parse_positive_u64_list(
        parsed.evidence_event_ids.as_deref(),
        "publish_social_fact",
        "evidence_event_ids",
    )?;
    let ttl_ticks =
        parse_optional_positive_u64(parsed.ttl_ticks, "publish_social_fact", "ttl_ticks")?;
    let stake = parse_social_stake(parsed.stake.as_ref(), "publish_social_fact")?;
    Ok(Action::PublishSocialFact {
        actor,
        schema_id,
        subject,
        object,
        claim,
        confidence_ppm,
        evidence_event_ids,
        ttl_ticks,
        stake,
    })
}

fn parse_challenge_social_fact(
    parsed: &LlmDecisionPayload,
    agent_id: &str,
) -> Result<Action, String> {
    let challenger = parse_owner_or_self(parsed.challenger.as_deref(), agent_id)?;
    let fact_id = parse_positive_u64(parsed.fact_id, "challenge_social_fact", "fact_id")?;
    let reason = parse_required_text(parsed.reason.as_deref(), "challenge_social_fact", "reason")?;
    let stake = parse_social_stake(parsed.stake.as_ref(), "challenge_social_fact")?;
    Ok(Action::ChallengeSocialFact {
        challenger,
        fact_id,
        reason,
        stake,
    })
}

fn parse_adjudicate_social_fact(
    parsed: &LlmDecisionPayload,
    agent_id: &str,
) -> Result<Action, String> {
    let adjudicator = parse_owner_or_self(parsed.adjudicator.as_deref(), agent_id)?;
    let fact_id = parse_positive_u64(parsed.fact_id, "adjudicate_social_fact", "fact_id")?;
    let adjudication_raw = parse_required_text(
        parsed.adjudication.as_deref(),
        "adjudicate_social_fact",
        "adjudication",
    )?;
    let decision =
        parse_social_adjudication_decision(adjudication_raw.as_str()).ok_or_else(|| {
            format!("adjudicate_social_fact invalid adjudication: {adjudication_raw}")
        })?;
    let notes = parse_required_text(parsed.notes.as_deref(), "adjudicate_social_fact", "notes")?;
    Ok(Action::AdjudicateSocialFact {
        adjudicator,
        fact_id,
        decision,
        notes,
    })
}

fn parse_revoke_social_fact(parsed: &LlmDecisionPayload, agent_id: &str) -> Result<Action, String> {
    let actor = parse_owner_or_self(parsed.actor.as_deref(), agent_id)?;
    let fact_id = parse_positive_u64(parsed.fact_id, "revoke_social_fact", "fact_id")?;
    let reason = parse_required_text(parsed.reason.as_deref(), "revoke_social_fact", "reason")?;
    Ok(Action::RevokeSocialFact {
        actor,
        fact_id,
        reason,
    })
}

fn parse_declare_social_edge(
    parsed: &LlmDecisionPayload,
    agent_id: &str,
) -> Result<Action, String> {
    let declarer = parse_owner_or_self(parsed.declarer.as_deref(), agent_id)?;
    let schema_id = parse_required_text(
        parsed.schema_id.as_deref(),
        "declare_social_edge",
        "schema_id",
    )?;
    let relation_kind = parse_required_text(
        parsed.relation_kind.as_deref(),
        "declare_social_edge",
        "relation_kind",
    )?;
    let from = parse_required_owner(
        parsed.from.as_deref(),
        "declare_social_edge",
        "from",
        agent_id,
    )?;
    let to = parse_required_owner(parsed.to.as_deref(), "declare_social_edge", "to", agent_id)?;
    let weight_bps = parsed.weight_bps.unwrap_or(0);
    if !(-10_000..=10_000).contains(&weight_bps) {
        return Err(format!(
            "declare_social_edge weight_bps out of range: {weight_bps} (expected -10000..=10000)"
        ));
    }
    let backing_fact_ids = parse_positive_u64_list(
        parsed.backing_fact_ids.as_deref(),
        "declare_social_edge",
        "backing_fact_ids",
    )?;
    let ttl_ticks =
        parse_optional_positive_u64(parsed.ttl_ticks, "declare_social_edge", "ttl_ticks")?;
    Ok(Action::DeclareSocialEdge {
        declarer,
        schema_id,
        relation_kind,
        from,
        to,
        weight_bps,
        backing_fact_ids,
        ttl_ticks,
    })
}

fn parse_required_owner(
    raw: Option<&str>,
    decision: &str,
    field_name: &str,
    agent_id: &str,
) -> Result<ResourceOwner, String> {
    let raw = raw.ok_or_else(|| format!("{decision} missing `{field_name}`"))?;
    parse_owner_spec(raw, agent_id)
}

fn parse_owner_or_self(raw: Option<&str>, agent_id: &str) -> Result<ResourceOwner, String> {
    match raw {
        Some(owner) => parse_owner_spec(owner, agent_id),
        None => Ok(ResourceOwner::Agent {
            agent_id: agent_id.to_string(),
        }),
    }
}

fn parse_agent_identity_or_self(
    raw: Option<&str>,
    decision: &str,
    field_name: &str,
    agent_id: &str,
) -> Result<String, String> {
    let owner = match raw {
        Some(value) => parse_owner_spec(value, agent_id)?,
        None => ResourceOwner::Agent {
            agent_id: agent_id.to_string(),
        },
    };
    match owner {
        ResourceOwner::Agent { agent_id } => Ok(agent_id),
        ResourceOwner::Location { .. } => Err(format!(
            "{decision} `{field_name}` must be self or agent:<id>"
        )),
    }
}

fn parse_required_text(
    raw: Option<&str>,
    decision: &str,
    field_name: &str,
) -> Result<String, String> {
    let raw = raw.ok_or_else(|| format!("{decision} missing `{field_name}`"))?;
    let normalized = raw.trim();
    if normalized.is_empty() {
        return Err(format!("{decision} `{field_name}` cannot be empty"));
    }
    Ok(normalized.to_string())
}

fn parse_positive_i64(value: Option<i64>, decision: &str, field_name: &str) -> Result<i64, String> {
    let value = value.ok_or_else(|| format!("{decision} missing `{field_name}`"))?;
    if value <= 0 {
        return Err(format!("{decision} requires positive {field_name}"));
    }
    Ok(value)
}

fn parse_non_negative_i64_with_default(
    value: Option<i64>,
    decision: &str,
    field_name: &str,
    default: i64,
) -> Result<i64, String> {
    let value = value.unwrap_or(default);
    if value < 0 {
        return Err(format!("{decision} requires non-negative {field_name}"));
    }
    Ok(value)
}

fn parse_hex_bytes(raw: Option<&str>, decision: &str, field_name: &str) -> Result<Vec<u8>, String> {
    let raw = parse_required_text(raw, decision, field_name)?;
    let normalized = raw.strip_prefix("0x").unwrap_or(raw.as_str());
    let bytes = hex::decode(normalized)
        .map_err(|_| format!("{decision} `{field_name}` must be valid hex"))?;
    if bytes.is_empty() {
        return Err(format!(
            "{decision} `{field_name}` cannot decode to empty bytes"
        ));
    }
    Ok(bytes)
}

fn parse_source_files(
    raw: Option<&BTreeMap<String, String>>,
    decision: &str,
    field_name: &str,
) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let Some(raw) = raw else {
        return Err(format!("{decision} missing `{field_name}`"));
    };
    if raw.is_empty() {
        return Err(format!("{decision} `{field_name}` cannot be empty"));
    }
    let mut files = BTreeMap::new();
    for (path, content) in raw {
        let normalized_path = path.trim();
        if normalized_path.is_empty() {
            return Err(format!("{decision} `{field_name}` contains empty path"));
        }
        if content.is_empty() {
            return Err(format!(
                "{decision} `{field_name}` contains empty content for path {}",
                normalized_path
            ));
        }
        files.insert(normalized_path.to_string(), content.as_bytes().to_vec());
    }
    Ok(files)
}

fn parse_positive_u64(value: Option<u64>, decision: &str, field_name: &str) -> Result<u64, String> {
    let value = value.ok_or_else(|| format!("{decision} missing `{field_name}`"))?;
    if value == 0 {
        return Err(format!("{decision} requires positive {field_name}"));
    }
    Ok(value)
}

fn parse_positive_u64_list(
    value: Option<&[u64]>,
    decision: &str,
    field_name: &str,
) -> Result<Vec<u64>, String> {
    let value = value.ok_or_else(|| format!("{decision} missing `{field_name}`"))?;
    if value.is_empty() {
        return Err(format!("{decision} `{field_name}` cannot be empty"));
    }
    if value.iter().any(|candidate| *candidate == 0) {
        return Err(format!("{decision} `{field_name}` must be positive"));
    }
    Ok(value.to_vec())
}

fn parse_optional_positive_u64(
    value: Option<u64>,
    decision: &str,
    field_name: &str,
) -> Result<Option<u64>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value == 0 {
        return Err(format!("{decision} `{field_name}` must be >= 1"));
    }
    Ok(Some(value))
}

fn parse_social_stake(
    payload: Option<&LlmSocialStakePayload>,
    decision: &str,
) -> Result<Option<SocialStake>, String> {
    let Some(payload) = payload else {
        return Ok(None);
    };
    let raw_kind = payload
        .kind
        .as_deref()
        .ok_or_else(|| format!("{decision} stake missing `kind`"))?;
    let kind = parse_resource_kind(raw_kind)
        .ok_or_else(|| format!("{decision} stake invalid kind: {raw_kind}"))?;
    let amount = payload
        .amount
        .ok_or_else(|| format!("{decision} stake missing `amount`"))?;
    if amount <= 0 {
        return Err(format!("{decision} stake requires positive amount"));
    }
    Ok(Some(SocialStake { kind, amount }))
}

fn parse_social_adjudication_decision(value: &str) -> Option<SocialAdjudicationDecision> {
    match value.trim().to_ascii_lowercase().as_str() {
        "confirm" => Some(SocialAdjudicationDecision::Confirm),
        "retract" => Some(SocialAdjudicationDecision::Retract),
        _ => None,
    }
}

fn parse_power_order_side(value: &str) -> Option<PowerOrderSide> {
    match value.trim().to_ascii_lowercase().as_str() {
        "buy" => Some(PowerOrderSide::Buy),
        "sell" => Some(PowerOrderSide::Sell),
        _ => None,
    }
}
