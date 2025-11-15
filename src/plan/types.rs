use crate::agent::AgentId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlanId(Uuid);

impl PlanId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PlanId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanStepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub description: String,
    pub status: PlanStepStatus,
    pub sub_steps: Vec<PlanStep>,
}

impl PlanStep {
    pub fn new(description: String) -> Self {
        Self {
            description,
            status: PlanStepStatus::Pending,
            sub_steps: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: PlanId,
    pub agent_id: AgentId,
    pub title: String,
    pub description: String,
    pub steps: Vec<PlanStep>,
}

impl Plan {
    pub fn new(agent_id: AgentId, title: String, description: String) -> Self {
        Self {
            id: PlanId::new(),
            agent_id,
            title,
            description,
            steps: Vec::new(),
        }
    }

    pub fn add_step(&mut self, step: PlanStep) {
        self.steps.push(step);
    }
}
