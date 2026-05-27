use memd_schema::{MemoryItem, SearchMemoryResponse};

use crate::runtime::retrieval_runtime::enum_label_kind;

use super::expansion::ExpansionPlan;

pub(crate) fn render_ceo_packet(
    query: &str,
    response: &SearchMemoryResponse,
    plan: &ExpansionPlan,
) -> String {
    let mut out = String::new();
    out.push_str("## CEO synthesis packet\n\n");
    out.push_str(&format!("- query: {}\n", query));
    out.push_str(&format!(
        "- expansion: {}\n\n",
        plan.stage_names().join(" -> ")
    ));

    if response.items.is_empty() {
        out.push_str("### Read\nNo durable records matched this query.\n\n");
        out.push_str("### Prize\nGap: no durable record states the desired outcome.\n\n");
        out.push_str("### Bottleneck\nGap: no durable record identifies the current blocker.\n\n");
        out.push_str("### Moves\nGap: no durable records support concrete moves.\n\n");
        out.push_str(
            "### Recommendation\nGap: insufficient durable evidence to recommend an action.\n\n",
        );
        out.push_str("### Proof\nNo matching durable memory records.\n");
        return out;
    }

    let top = response.items.iter().take(3).collect::<Vec<_>>();
    let first = top[0];

    out.push_str("### Read\n");
    for item in &top {
        out.push_str(&format!("- {}\n", compact_item_line(item)));
    }
    out.push('\n');

    out.push_str("### Prize\n");
    out.push_str(&format!(
        "Best supported objective from durable memory: {}\n\n",
        sanitized_content(first)
    ));

    out.push_str("### Bottleneck\n");
    if let Some(item) = response
        .items
        .iter()
        .find(|item| is_bottleneck_candidate(item))
    {
        out.push_str(&format!(
            "Supported blocker/risk: {}\n\n",
            sanitized_content(item)
        ));
    } else {
        out.push_str("Gap: matched records do not explicitly identify a blocker.\n\n");
    }

    out.push_str("### Moves\n");
    for item in &top {
        out.push_str(&format!(
            "- Use [{}] as an input: {}\n",
            short_id(item),
            sanitized_content(item)
        ));
    }
    out.push('\n');

    out.push_str("### Recommendation\n");
    out.push_str(
        "Act only on the supported records above; where a section is marked Gap, gather or write memory before relying on it.\n\n",
    );

    out.push_str("### Proof\n");
    for item in response
        .items
        .iter()
        .take(if plan.forensics { 8 } else { 3 })
    {
        out.push_str(&format!("- {}\n", compact_item_line(item)));
    }

    out
}

fn compact_item_line(item: &MemoryItem) -> String {
    format!(
        "[{}] {} ({}, {:.2})",
        short_id(item),
        sanitized_content(item),
        enum_label_kind(item.kind),
        item.confidence
    )
}

fn sanitized_content(item: &MemoryItem) -> String {
    item.content.replace('\n', " ")
}

fn short_id(item: &MemoryItem) -> String {
    item.id.to_string().chars().take(8).collect()
}

fn is_bottleneck_candidate(item: &MemoryItem) -> bool {
    let content = item.content.to_ascii_lowercase();
    content.contains("block")
        || content.contains("risk")
        || content.contains("bottleneck")
        || content.contains("issue")
        || content.contains("constraint")
}
