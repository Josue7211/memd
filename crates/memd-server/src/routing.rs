use memd_schema::{MemoryScope, RetrievalIntent, RetrievalRoute};

#[derive(Debug, Clone, Copy)]
pub struct RetrievalPlan {
    pub route: RetrievalRoute,
    pub intent: RetrievalIntent,
}

impl RetrievalPlan {
    pub fn resolve(route: Option<RetrievalRoute>, intent: Option<RetrievalIntent>) -> Self {
        let intent = intent.unwrap_or(RetrievalIntent::General);
        let route = match route.unwrap_or(RetrievalRoute::Auto) {
            RetrievalRoute::Auto => default_route_for_intent(intent),
            other => other,
        };

        Self { route, intent }
    }

    pub fn scopes(self) -> Vec<MemoryScope> {
        match self.route {
            RetrievalRoute::Auto | RetrievalRoute::All => vec![
                MemoryScope::Local,
                MemoryScope::Synced,
                MemoryScope::Project,
                MemoryScope::Global,
            ],
            RetrievalRoute::LocalOnly => vec![MemoryScope::Local],
            RetrievalRoute::SyncedOnly => vec![MemoryScope::Synced],
            RetrievalRoute::ProjectOnly => vec![MemoryScope::Project],
            RetrievalRoute::GlobalOnly => vec![MemoryScope::Global],
            RetrievalRoute::LocalFirst => vec![
                MemoryScope::Local,
                MemoryScope::Synced,
                MemoryScope::Project,
                MemoryScope::Global,
            ],
            RetrievalRoute::SyncedFirst => vec![
                MemoryScope::Synced,
                MemoryScope::Project,
                MemoryScope::Local,
                MemoryScope::Global,
            ],
            RetrievalRoute::ProjectFirst => vec![
                MemoryScope::Project,
                MemoryScope::Synced,
                MemoryScope::Local,
                MemoryScope::Global,
            ],
            RetrievalRoute::GlobalFirst => vec![
                MemoryScope::Global,
                MemoryScope::Project,
                MemoryScope::Synced,
                MemoryScope::Local,
            ],
        }
    }

    pub fn allows(self, scope: MemoryScope) -> bool {
        match self.route {
            RetrievalRoute::LocalOnly => scope == MemoryScope::Local,
            RetrievalRoute::SyncedOnly => scope == MemoryScope::Synced,
            RetrievalRoute::ProjectOnly => scope == MemoryScope::Project,
            RetrievalRoute::GlobalOnly => scope == MemoryScope::Global,
            _ => true,
        }
    }

    pub fn scope_rank_bonus(self, scope: MemoryScope) -> f32 {
        let scopes = self.scopes();
        let Some(index) = scopes.iter().position(|candidate| *candidate == scope) else {
            return -2.0;
        };
        ((scopes.len() - index) as f32) * 0.3
    }

    pub fn intent_scope_bonus(self, scope: MemoryScope) -> f32 {
        match self.intent {
            RetrievalIntent::General => match scope {
                MemoryScope::Local => 0.4,
                MemoryScope::Synced => 0.55,
                MemoryScope::Project => 0.7,
                MemoryScope::Global => 0.5,
            },
            RetrievalIntent::CurrentTask => match scope {
                MemoryScope::Local => 1.1,
                MemoryScope::Synced => 0.95,
                MemoryScope::Project => 0.5,
                MemoryScope::Global => -0.2,
            },
            RetrievalIntent::Decision => match scope {
                MemoryScope::Project => 1.1,
                MemoryScope::Synced => 0.8,
                MemoryScope::Local => 0.3,
                MemoryScope::Global => 0.1,
            },
            RetrievalIntent::Runbook => match scope {
                MemoryScope::Project => 1.05,
                MemoryScope::Global => 0.95,
                MemoryScope::Synced => 0.4,
                MemoryScope::Local => 0.2,
            },
            RetrievalIntent::Procedural => match scope {
                MemoryScope::Project => 1.08,
                MemoryScope::Global => 0.92,
                MemoryScope::Synced => 0.55,
                MemoryScope::Local => 0.25,
            },
            RetrievalIntent::SelfModel => match scope {
                MemoryScope::Local => 1.0,
                MemoryScope::Project => 0.9,
                MemoryScope::Synced => 0.7,
                MemoryScope::Global => 0.35,
            },
            RetrievalIntent::Topology => match scope {
                MemoryScope::Project => 1.0,
                MemoryScope::Global => 0.9,
                MemoryScope::Synced => 0.45,
                MemoryScope::Local => 0.1,
            },
            RetrievalIntent::Preference => match scope {
                MemoryScope::Global => 1.05,
                MemoryScope::Project => 0.75,
                MemoryScope::Synced => 0.5,
                MemoryScope::Local => 0.2,
            },
            RetrievalIntent::Fact => match scope {
                MemoryScope::Project => 0.95,
                MemoryScope::Synced => 0.7,
                MemoryScope::Global => 0.55,
                MemoryScope::Local => 0.25,
            },
            RetrievalIntent::Pattern => match scope {
                MemoryScope::Global => 1.0,
                MemoryScope::Project => 0.8,
                MemoryScope::Synced => 0.45,
                MemoryScope::Local => 0.15,
            },
        }
    }
}

fn default_route_for_intent(intent: RetrievalIntent) -> RetrievalRoute {
    match intent {
        RetrievalIntent::General | RetrievalIntent::Fact => RetrievalRoute::All,
        RetrievalIntent::CurrentTask => RetrievalRoute::LocalFirst,
        RetrievalIntent::Decision
        | RetrievalIntent::Runbook
        | RetrievalIntent::Procedural
        | RetrievalIntent::Topology => {
            RetrievalRoute::ProjectFirst
        }
        RetrievalIntent::Preference | RetrievalIntent::Pattern => RetrievalRoute::GlobalFirst,
        RetrievalIntent::SelfModel => RetrievalRoute::LocalFirst,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_task_defaults_to_local_first() {
        let plan = RetrievalPlan::resolve(None, Some(RetrievalIntent::CurrentTask));
        assert_eq!(plan.route, RetrievalRoute::LocalFirst);
        assert_eq!(
            plan.scopes(),
            vec![
                MemoryScope::Local,
                MemoryScope::Synced,
                MemoryScope::Project,
                MemoryScope::Global
            ]
        );
    }

    #[test]
    fn preference_defaults_to_global_first() {
        let plan = RetrievalPlan::resolve(None, Some(RetrievalIntent::Preference));
        assert_eq!(plan.route, RetrievalRoute::GlobalFirst);
        assert!(
            plan.intent_scope_bonus(MemoryScope::Global)
                > plan.intent_scope_bonus(MemoryScope::Local)
        );
    }

    #[test]
    fn explicit_only_route_filters_scope() {
        let plan = RetrievalPlan::resolve(Some(RetrievalRoute::ProjectOnly), None);
        assert!(plan.allows(MemoryScope::Project));
        assert!(!plan.allows(MemoryScope::Global));
    }

    #[test]
    fn procedural_defaults_to_project_first() {
        let plan = RetrievalPlan::resolve(None, Some(RetrievalIntent::Procedural));
        assert_eq!(plan.route, RetrievalRoute::ProjectFirst);
        assert!(
            plan.intent_scope_bonus(MemoryScope::Project)
                > plan.intent_scope_bonus(MemoryScope::Local)
        );
    }

    #[test]
    fn self_model_defaults_to_local_first() {
        let plan = RetrievalPlan::resolve(None, Some(RetrievalIntent::SelfModel));
        assert_eq!(plan.route, RetrievalRoute::LocalFirst);
        assert!(
            plan.intent_scope_bonus(MemoryScope::Local)
                > plan.intent_scope_bonus(MemoryScope::Global)
        );
    }
}
