use super::*;

pub(super) fn validate_recipe_material_stacks(
    label: &str,
    stacks: &[MaterialStack],
) -> Result<(), String> {
    for stack in stacks {
        if stack.kind.trim().is_empty() || stack.amount <= 0 {
            return Err(format!(
                "{label} stack must have a non-empty kind and amount > 0"
            ));
        }
    }
    Ok(())
}

pub(super) fn aggregate_recipe_material_stacks(
    stacks: &[MaterialStack],
) -> Result<BTreeMap<String, i64>, String> {
    let mut totals: BTreeMap<String, i64> = BTreeMap::new();
    for stack in stacks {
        let entry = totals.entry(stack.kind.clone()).or_insert(0);
        *entry = entry
            .checked_add(stack.amount)
            .ok_or_else(|| format!("material amount overflow for kind={}", stack.kind))?;
    }
    Ok(totals)
}

pub(super) fn validate_recipe_output_capacity(
    ledgers: &BTreeMap<MaterialLedgerId, BTreeMap<String, i64>>,
    ledger: &MaterialLedgerId,
    produce: &[MaterialStack],
    byproducts: &[MaterialStack],
) -> Result<(), String> {
    validate_recipe_material_stacks("produce", produce)?;
    validate_recipe_material_stacks("byproduct", byproducts)?;

    let mut totals = aggregate_recipe_material_stacks(produce)?;
    for (kind, amount) in aggregate_recipe_material_stacks(byproducts)? {
        let entry = totals.entry(kind).or_insert(0);
        *entry = entry
            .checked_add(amount)
            .ok_or_else(|| "combined output amount overflow".to_string())?;
    }

    let current_balances = ledgers.get(ledger);
    for (kind, amount) in totals {
        let current = current_balances
            .and_then(|balances| balances.get(&kind))
            .copied()
            .unwrap_or(0);
        if current < 0 {
            return Err(format!(
                "negative existing material balance: ledger={ledger} kind={kind} balance={current}"
            ));
        }
        current.checked_add(amount).ok_or_else(|| {
            format!("material output balance overflow: ledger={ledger} kind={kind}")
        })?;
    }
    Ok(())
}
