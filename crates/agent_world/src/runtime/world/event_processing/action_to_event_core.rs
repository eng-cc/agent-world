use super::*;

impl World {
    pub(super) fn action_to_event_core(
        &self,
        action_id: ActionId,
        action: &Action,
    ) -> Result<WorldEventBody, WorldError> {
        match action {
            Action::RegisterAgent { agent_id, pos } => {
                if self.state.agents.contains_key(agent_id) {
                    Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentAlreadyExists {
                            agent_id: agent_id.clone(),
                        },
                    }))
                } else {
                    Ok(WorldEventBody::Domain(DomainEvent::AgentRegistered {
                        agent_id: agent_id.clone(),
                        pos: *pos,
                    }))
                }
            }
            Action::MoveAgent { agent_id, to } => match self.state.agents.get(agent_id) {
                Some(cell) => Ok(WorldEventBody::Domain(DomainEvent::AgentMoved {
                    agent_id: agent_id.clone(),
                    from: cell.state.pos,
                    to: *to,
                })),
                None => Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::AgentNotFound {
                        agent_id: agent_id.clone(),
                    },
                })),
            },
            Action::QueryObservation { agent_id } => {
                if self.state.agents.contains_key(agent_id) {
                    Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec!["observation requires rule module".to_string()],
                        },
                    }))
                } else {
                    Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: agent_id.clone(),
                        },
                    }))
                }
            }
            Action::EmitObservation { observation } => {
                Ok(WorldEventBody::Domain(DomainEvent::Observation {
                    observation: observation.clone(),
                }))
            }
            Action::BodyAction { agent_id, .. } => {
                if self.state.agents.contains_key(agent_id) {
                    Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec!["body action requires body module".to_string()],
                        },
                    }))
                } else {
                    Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: agent_id.clone(),
                        },
                    }))
                }
            }
            Action::EmitBodyAttributes {
                agent_id,
                view,
                reason,
            } => {
                let Some(cell) = self.state.agents.get(agent_id) else {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: agent_id.clone(),
                        },
                    }));
                };
                if let Err(reason) = validate_body_kernel_view(&cell.state.body_view, view) {
                    return Ok(WorldEventBody::Domain(
                        DomainEvent::BodyAttributesRejected {
                            agent_id: agent_id.clone(),
                            reason,
                        },
                    ));
                }
                Ok(WorldEventBody::Domain(DomainEvent::BodyAttributesUpdated {
                    agent_id: agent_id.clone(),
                    view: view.clone(),
                    reason: reason.clone(),
                }))
            }
            Action::ExpandBodyInterface {
                agent_id,
                interface_module_item_id,
            } => Ok(evaluate_expand_body_interface(
                self,
                action_id,
                agent_id,
                interface_module_item_id,
            )),
            Action::DeployModuleArtifact { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec![
                            "deploy_module_artifact requires runtime action loop".to_string()
                        ],
                    },
                }))
            }
            Action::CompileModuleArtifactFromSource { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec![
                            "compile_module_artifact_from_source requires runtime action loop"
                                .to_string(),
                        ],
                    },
                }))
            }
            Action::InstallModuleFromArtifact { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec![
                            "install_module_from_artifact requires runtime action loop".to_string()
                        ],
                    },
                }))
            }
            Action::InstallModuleToTargetFromArtifact { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec![
                            "install_module_to_target_from_artifact requires runtime action loop"
                                .to_string(),
                        ],
                    },
                }))
            }
            Action::UpgradeModuleFromArtifact { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec![
                            "upgrade_module_from_artifact requires runtime action loop".to_string()
                        ],
                    },
                }))
            }
            Action::ListModuleArtifactForSale { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec!["list_module_artifact_for_sale requires runtime action loop"
                            .to_string()],
                    },
                }))
            }
            Action::BuyModuleArtifact { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec!["buy_module_artifact requires runtime action loop".to_string()],
                    },
                }))
            }
            Action::DelistModuleArtifact { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec![
                            "delist_module_artifact requires runtime action loop".to_string()
                        ],
                    },
                }))
            }
            Action::DestroyModuleArtifact { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec![
                            "destroy_module_artifact requires runtime action loop".to_string()
                        ],
                    },
                }))
            }
            Action::PlaceModuleArtifactBid { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec![
                            "place_module_artifact_bid requires runtime action loop".to_string()
                        ],
                    },
                }))
            }
            Action::CancelModuleArtifactBid { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec![
                            "cancel_module_artifact_bid requires runtime action loop".to_string()
                        ],
                    },
                }))
            }
            Action::TransferResource {
                from_agent_id,
                to_agent_id,
                ..
            } => {
                if !self.state.agents.contains_key(from_agent_id) {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: from_agent_id.clone(),
                        },
                    }));
                }
                if !self.state.agents.contains_key(to_agent_id) {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: to_agent_id.clone(),
                        },
                    }));
                }
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec!["transfer requires rule module".to_string()],
                    },
                }))
            }
            Action::RedeemPower {
                node_id,
                target_agent_id,
                redeem_credits,
                nonce,
            } => Ok(WorldEventBody::Domain(self.evaluate_redeem_power_action(
                node_id.as_str(),
                target_agent_id.as_str(),
                *redeem_credits,
                *nonce,
                None,
            ))),
            Action::RedeemPowerSigned {
                node_id,
                target_agent_id,
                redeem_credits,
                nonce,
                signer_node_id,
                signature,
            } => Ok(WorldEventBody::Domain(self.evaluate_redeem_power_action(
                node_id.as_str(),
                target_agent_id.as_str(),
                *redeem_credits,
                *nonce,
                Some((signer_node_id.as_str(), signature.as_str())),
            ))),
            Action::ApplyNodePointsSettlementSigned {
                report,
                signer_node_id,
                mint_records,
            } => Ok(WorldEventBody::Domain(
                self.evaluate_apply_node_points_settlement_action(
                    action_id,
                    report,
                    signer_node_id.as_str(),
                    mint_records.as_slice(),
                ),
            )),
            Action::TransferMaterial {
                requester_agent_id,
                from_ledger,
                to_ledger,
                kind,
                amount,
                distance_km,
            } => {
                if !self.state.agents.contains_key(requester_agent_id) {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: requester_agent_id.clone(),
                        },
                    }));
                }
                if from_ledger == to_ledger {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec!["from_ledger and to_ledger cannot be the same".to_string()],
                        },
                    }));
                }
                if kind.trim().is_empty() {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec!["material kind cannot be empty".to_string()],
                        },
                    }));
                }
                if *amount <= 0 {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::InvalidAmount { amount: *amount },
                    }));
                }
                if *distance_km < 0 {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec!["distance_km must be >= 0".to_string()],
                        },
                    }));
                }
                if *distance_km > MATERIAL_TRANSFER_MAX_DISTANCE_KM {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::MaterialTransferDistanceExceeded {
                            distance_km: *distance_km,
                            max_distance_km: MATERIAL_TRANSFER_MAX_DISTANCE_KM,
                        },
                    }));
                }
                let available = self.ledger_material_balance(from_ledger, kind.as_str());
                if available < *amount {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::InsufficientMaterial {
                            material_kind: kind.clone(),
                            requested: *amount,
                            available,
                        },
                    }));
                }

                if *distance_km == 0 {
                    return Ok(WorldEventBody::Domain(DomainEvent::MaterialTransferred {
                        requester_agent_id: requester_agent_id.clone(),
                        from_ledger: from_ledger.clone(),
                        to_ledger: to_ledger.clone(),
                        kind: kind.clone(),
                        amount: *amount,
                        distance_km: *distance_km,
                    }));
                }

                if self.state.pending_material_transits.len() >= MATERIAL_TRANSFER_MAX_INFLIGHT {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::MaterialTransitCapacityExceeded {
                            in_flight: self.state.pending_material_transits.len(),
                            max_in_flight: MATERIAL_TRANSFER_MAX_INFLIGHT,
                        },
                    }));
                }

                let transit_ticks = ((*distance_km + MATERIAL_TRANSFER_SPEED_KM_PER_TICK - 1)
                    / MATERIAL_TRANSFER_SPEED_KM_PER_TICK)
                    .max(1) as u64;
                let ready_at = self.state.time.saturating_add(transit_ticks);
                Ok(WorldEventBody::Domain(
                    DomainEvent::MaterialTransitStarted {
                        job_id: action_id,
                        requester_agent_id: requester_agent_id.clone(),
                        from_ledger: from_ledger.clone(),
                        to_ledger: to_ledger.clone(),
                        kind: kind.clone(),
                        amount: *amount,
                        distance_km: *distance_km,
                        loss_bps: MATERIAL_TRANSFER_LOSS_PER_KM_BPS,
                        ready_at,
                    },
                ))
            }
            _ => unreachable!("action_to_event_core received unsupported action variant"),
        }
    }
}
