use chrono::Utc;
use memd_schema::{
    AtlasExploreRequest, AtlasExploreResponse, AtlasLink, AtlasLinkKind, AtlasNode, AtlasRegion,
    AtlasRegionsRequest, AtlasRegionsResponse, MemoryItem, MemoryKind, MemoryStatus,
};
use rusqlite::params;
use uuid::Uuid;

use crate::SqliteStore;

impl SqliteStore {
    pub(crate) fn list_atlas_regions(
        &self,
        req: &AtlasRegionsRequest,
    ) -> anyhow::Result<AtlasRegionsResponse> {
        let conn = self.connect()?;
        let mut sql = String::from(
            "SELECT id, payload_json FROM atlas_regions WHERE 1=1",
        );
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(project) = &req.project {
            sql.push_str(" AND project = ?");
            bind_values.push(project.clone());
        }
        if let Some(namespace) = &req.namespace {
            sql.push_str(" AND namespace = ?");
            bind_values.push(namespace.clone());
        }
        if let Some(lane) = &req.lane {
            sql.push_str(" AND lane = ?");
            bind_values.push(lane.clone());
        }
        sql.push_str(" ORDER BY updated_at DESC");
        let limit = req.limit.unwrap_or(20);
        sql.push_str(&format!(" LIMIT {limit}"));

        let mut stmt = conn.prepare(&sql)?;
        let params: Vec<&dyn rusqlite::ToSql> = bind_values
            .iter()
            .map(|v| v as &dyn rusqlite::ToSql)
            .collect();
        let regions = stmt
            .query_map(params.as_slice(), |row| {
                let payload: String = row.get(1)?;
                Ok(payload)
            })?
            .filter_map(|r| r.ok())
            .filter_map(|payload| serde_json::from_str::<AtlasRegion>(&payload).ok())
            .collect();

        Ok(AtlasRegionsResponse { regions })
    }

    pub(crate) fn upsert_atlas_region(&self, region: &AtlasRegion) -> anyhow::Result<()> {
        let conn = self.connect()?;
        let payload = serde_json::to_string(region)?;
        conn.execute(
            r#"
            INSERT INTO atlas_regions (id, name, project, namespace, lane, auto_generated, created_at, updated_at, payload_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(id) DO UPDATE SET
              name = excluded.name,
              lane = excluded.lane,
              auto_generated = excluded.auto_generated,
              updated_at = excluded.updated_at,
              payload_json = excluded.payload_json
            "#,
            params![
                region.id.to_string(),
                region.name,
                region.project,
                region.namespace,
                region.lane,
                region.auto_generated as i32,
                region.created_at.to_rfc3339(),
                region.updated_at.to_rfc3339(),
                payload,
            ],
        )?;
        Ok(())
    }

    pub(crate) fn set_region_members(
        &self,
        region_id: Uuid,
        memory_ids: &[Uuid],
    ) -> anyhow::Result<()> {
        let conn = self.connect()?;
        let region_str = region_id.to_string();
        conn.execute(
            "DELETE FROM atlas_region_members WHERE region_id = ?1",
            params![region_str],
        )?;
        let mut stmt = conn.prepare(
            "INSERT OR IGNORE INTO atlas_region_members (region_id, memory_id) VALUES (?1, ?2)",
        )?;
        for id in memory_ids {
            stmt.execute(params![region_str, id.to_string()])?;
        }
        Ok(())
    }

    pub(crate) fn get_region_member_ids(&self, region_id: Uuid) -> anyhow::Result<Vec<Uuid>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT memory_id FROM atlas_region_members WHERE region_id = ?1",
        )?;
        let ids = stmt
            .query_map(params![region_id.to_string()], |row| {
                row.get::<_, String>(0)
            })?
            .filter_map(|r| r.ok())
            .filter_map(|s| s.parse::<Uuid>().ok())
            .collect();
        Ok(ids)
    }

    pub(crate) fn explore_atlas(
        &self,
        req: &AtlasExploreRequest,
    ) -> anyhow::Result<AtlasExploreResponse> {
        let limit = req.limit.unwrap_or(20);
        let depth = req.depth.unwrap_or(1);

        // If a region is specified, load it and its members
        let (region, seed_ids) = if let Some(region_id) = req.region_id {
            let region = self.get_atlas_region_by_id(region_id)?;
            let member_ids = self.get_region_member_ids(region_id)?;
            (region, member_ids)
        } else if let Some(node_id) = req.node_id {
            (None, vec![node_id])
        } else {
            // No anchor — generate regions on the fly for the project
            let regions = self.generate_regions_for_project(
                req.project.as_deref(),
                req.namespace.as_deref(),
                req.lane.as_deref(),
            )?;
            return Ok(AtlasExploreResponse {
                region: regions.first().cloned(),
                nodes: Vec::new(),
                links: Vec::new(),
                trails: Vec::new(),
                truncated: false,
            });
        };

        // Load memory items for seed IDs
        let mut nodes = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for id in &seed_ids {
            if let Some(item) = self.get(*id)? {
                if !passes_pivot_filters(&item, req) {
                    continue;
                }
                if seen.insert(item.id) {
                    nodes.push(item_to_atlas_node(&item, region.as_ref().map(|r| r.id), 0));
                }
            }
        }

        // Neighborhood expansion via entity links
        let mut links = Vec::new();
        if depth > 0 {
            for seed_id in &seed_ids {
                let entity_links = self.get_entity_links_for_item(*seed_id)?;
                for el in entity_links {
                    let neighbor_id = if el.from_entity_id == *seed_id {
                        el.to_entity_id
                    } else {
                        el.from_entity_id
                    };
                    // Try to find memory items linked to this entity
                    if let Some(neighbor_item) = self.get(neighbor_id)? {
                        if !passes_pivot_filters(&neighbor_item, req) {
                            continue;
                        }
                        if seen.insert(neighbor_item.id) {
                            nodes.push(item_to_atlas_node(
                                &neighbor_item,
                                region.as_ref().map(|r| r.id),
                                1,
                            ));
                        }
                        links.push(AtlasLink {
                            from_node_id: *seed_id,
                            to_node_id: neighbor_item.id,
                            link_kind: entity_relation_to_atlas_link(el.relation_kind),
                            weight: el.confidence,
                            label: el.note.clone(),
                        });
                    }
                }
            }
        }

        let truncated = nodes.len() > limit;
        nodes.truncate(limit);

        Ok(AtlasExploreResponse {
            region,
            nodes,
            links,
            trails: Vec::new(),
            truncated,
        })
    }

    fn get_atlas_region_by_id(&self, id: Uuid) -> anyhow::Result<Option<AtlasRegion>> {
        let conn = self.connect()?;
        let payload: Option<String> = conn
            .query_row(
                "SELECT payload_json FROM atlas_regions WHERE id = ?1",
                params![id.to_string()],
                |row| row.get(0),
            )
            .ok();
        match payload {
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
            None => Ok(None),
        }
    }

    pub(crate) fn generate_regions_for_project(
        &self,
        project: Option<&str>,
        namespace: Option<&str>,
        lane_filter: Option<&str>,
    ) -> anyhow::Result<Vec<AtlasRegion>> {
        let items = self.list()?;
        let now = Utc::now();
        let mut buckets: std::collections::HashMap<String, Vec<MemoryItem>> =
            std::collections::HashMap::new();

        for item in items {
            if item.status != MemoryStatus::Active {
                continue;
            }
            if let Some(p) = project {
                if item.project.as_deref() != Some(p) {
                    continue;
                }
            }
            if let Some(ns) = namespace {
                if item.namespace.as_deref() != Some(ns) {
                    continue;
                }
            }

            let bucket_key = region_bucket_key(&item, lane_filter);
            if let Some(key) = bucket_key {
                buckets.entry(key).or_default().push(item);
            }
        }

        let mut regions = Vec::new();
        for (key, members) in &buckets {
            if members.len() < 2 {
                continue;
            }
            let region_id = deterministic_region_id(project, namespace, key);
            let region = AtlasRegion {
                id: region_id,
                name: key.clone(),
                description: None,
                project: project.map(String::from),
                namespace: namespace.map(String::from),
                lane: lane_filter.map(String::from),
                auto_generated: true,
                node_count: members.len(),
                tags: Vec::new(),
                created_at: now,
                updated_at: now,
            };
            // Persist the generated region
            let _ = self.upsert_atlas_region(&region);
            let member_ids: Vec<Uuid> = members.iter().map(|m| m.id).collect();
            let _ = self.set_region_members(region_id, &member_ids);
            regions.push(region);
        }

        regions.sort_by(|a, b| b.node_count.cmp(&a.node_count));
        Ok(regions)
    }

    fn get_entity_links_for_item(
        &self,
        item_id: Uuid,
    ) -> anyhow::Result<Vec<memd_schema::MemoryEntityLinkRecord>> {
        let conn = self.connect()?;
        let id_str = item_id.to_string();
        let mut stmt = conn.prepare(
            "SELECT payload_json FROM memory_entity_links WHERE from_entity_id = ?1 OR to_entity_id = ?1",
        )?;
        let links = stmt
            .query_map(params![id_str], |row| row.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .filter_map(|json| serde_json::from_str(&json).ok())
            .collect();
        Ok(links)
    }
}

fn deterministic_region_id(
    project: Option<&str>,
    namespace: Option<&str>,
    key: &str,
) -> Uuid {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    project.hash(&mut hasher);
    namespace.hash(&mut hasher);
    key.hash(&mut hasher);
    let hash = hasher.finish();
    let bytes = hash.to_le_bytes();
    let mut uuid_bytes = [0u8; 16];
    uuid_bytes[..8].copy_from_slice(&bytes);
    uuid_bytes[8..16].copy_from_slice(&bytes);
    // Set version 4 and variant bits for valid UUID
    uuid_bytes[6] = (uuid_bytes[6] & 0x0f) | 0x40;
    uuid_bytes[8] = (uuid_bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(uuid_bytes)
}

fn region_bucket_key(item: &MemoryItem, lane_filter: Option<&str>) -> Option<String> {
    // Group by lane_id if present
    if let Some(lane) = item_lane(item) {
        if let Some(filter) = lane_filter {
            if lane != filter {
                return None;
            }
        }
        return Some(lane);
    }

    // Fall back to grouping by kind
    let kind_label = match item.kind {
        MemoryKind::Fact | MemoryKind::Constraint | MemoryKind::LiveTruth => "facts",
        MemoryKind::Decision | MemoryKind::Preference => "decisions",
        MemoryKind::Runbook | MemoryKind::Procedural => "procedures",
        MemoryKind::Status => "continuity",
        MemoryKind::Pattern => "patterns",
        MemoryKind::SelfModel | MemoryKind::Topology => "model",
    };

    if lane_filter.is_some() {
        return None; // lane filter set but item has no lane
    }

    Some(kind_label.to_string())
}

fn item_lane(item: &MemoryItem) -> Option<String> {
    // Check tags for lane markers
    for tag in &item.tags {
        if let Some(lane) = tag.strip_prefix("lane:") {
            return Some(lane.to_string());
        }
    }
    // Check source_path for lane hints
    if let Some(path) = item.source_path.as_deref() {
        for prefix in &["design", "architecture", "research", "workflow", "preference", "inspiration"] {
            if path.contains(prefix) {
                return Some(prefix.to_string());
            }
        }
    }
    None
}

fn passes_pivot_filters(item: &MemoryItem, req: &AtlasExploreRequest) -> bool {
    if let Some(min_trust) = req.min_trust {
        if item.confidence < min_trust {
            return false;
        }
    }
    if let Some(pivot_kind) = req.pivot_kind {
        if item.kind != pivot_kind {
            return false;
        }
    }
    if let Some(project) = &req.project {
        if item.project.as_deref() != Some(project) {
            return false;
        }
    }
    if let Some(namespace) = &req.namespace {
        if item.namespace.as_deref() != Some(namespace) {
            return false;
        }
    }
    true
}

fn item_to_atlas_node(item: &MemoryItem, region_id: Option<Uuid>, depth: usize) -> AtlasNode {
    AtlasNode {
        id: item.id,
        region_id,
        memory_id: item.id,
        entity_id: None,
        label: compact_label(&item.content),
        kind: item.kind,
        stage: item.stage,
        lane: item_lane(item),
        confidence: item.confidence,
        salience: item.confidence,
        depth,
        tags: item.tags.clone(),
    }
}

fn compact_label(content: &str) -> String {
    let first_line = content.lines().next().unwrap_or(content);
    if first_line.len() <= 80 {
        first_line.to_string()
    } else {
        format!("{}...", &first_line[..77])
    }
}

fn entity_relation_to_atlas_link(
    kind: memd_schema::EntityRelationKind,
) -> AtlasLinkKind {
    match kind {
        memd_schema::EntityRelationKind::SameAs => AtlasLinkKind::Semantic,
        memd_schema::EntityRelationKind::DerivedFrom => AtlasLinkKind::Causal,
        memd_schema::EntityRelationKind::Supersedes => AtlasLinkKind::Corrective,
        memd_schema::EntityRelationKind::Contradicts => AtlasLinkKind::Corrective,
        memd_schema::EntityRelationKind::Related => AtlasLinkKind::Semantic,
    }
}
