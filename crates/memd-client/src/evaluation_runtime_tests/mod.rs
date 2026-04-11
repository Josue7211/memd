    use super::*;
    use std::sync::{Arc, Mutex, OnceLock};

    use crate::render::{
        render_agent_zero_harness_pack_markdown, render_claude_code_harness_pack_markdown,
        render_codex_harness_pack_markdown, render_command_catalog_markdown,
        render_command_catalog_summary, render_hermes_harness_pack_markdown,
        render_openclaw_harness_pack_markdown, render_opencode_harness_pack_markdown,
    };
    use axum::{
        Json, Router,
        extract::{Query, State},
        http::StatusCode,
        routing::{get, post},
    };
    use memd_schema::{
        BenchmarkEvidenceSummary, BenchmarkFeatureRecord, BenchmarkGateDecision,
        BenchmarkSubjectMetrics, ContinuityJourneyReport, HiveClaimAcquireRequest, HiveClaimRecord,
        HiveClaimReleaseRequest, HiveClaimTransferRequest, HiveClaimsRequest, HiveClaimsResponse,
        HiveCoordinationInboxResponse, HiveCoordinationReceiptRecord,
        HiveCoordinationReceiptRequest, HiveCoordinationReceiptsResponse, HiveMessageAckRequest,
        HiveMessageInboxRequest, HiveMessageRecord, HiveMessageSendRequest, HiveMessagesResponse,
        HiveTaskRecord, SkillPolicyActivationRecord, SkillPolicyApplyReceipt,
        SkillPolicyApplyReceiptsRequest, SkillPolicyApplyReceiptsResponse, SkillPolicyApplyRequest,
        SkillPolicyApplyResponse, VerifierAssertionRecord, VerifierStepRecord,
    };

    #[path = "evaluation_runtime_tests_support.rs"]
    mod evaluation_runtime_tests_support;
    use evaluation_runtime_tests_support::*;

    #[path = "evaluation_runtime_tests_tail.rs"]
    mod evaluation_runtime_tests_tail;

