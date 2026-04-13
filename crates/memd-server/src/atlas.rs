use chrono::Utc;
use memd_schema::{
    AtlasExpandRequest, AtlasExpandResponse, AtlasExploreRequest, AtlasExploreResponse, AtlasLink,
    AtlasLinkKind, AtlasListTrailsRequest, AtlasListTrailsResponse, AtlasNode, AtlasRegion,
    AtlasRegionsRequest, AtlasRegionsResponse, AtlasRenameRegionRequest,
    AtlasRenameRegionResponse, AtlasSaveTrailRequest, AtlasSaveTrailResponse, AtlasSavedTrail,
    MemoryEventRecord, MemoryItem, MemoryKind, MemoryStatus,
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
            .filter_map(|r| r.inspect_err(|e| eprintln!("warn: atlas region row read: {e}")).ok())
            .filter_map(|payload| serde_json::from_str::<AtlasRegion>(&payload).inspect_err(|e| eprintln!("warn: atlas region json parse: {e}")).ok())
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
            .filter_map(|r| r.inspect_err(|e| eprintln!("warn: region member row read: {e}")).ok())
            .filter_map(|s| s.parse::<Uuid>().inspect_err(|e| eprintln!("warn: region member uuid parse: {e}")).ok())
            .collect();
        Ok(ids)
    }

    pub(crate) fn explore_atlas(
        &self,
        req: &AtlasExploreRequest,
    ) -> anyhow::Result<AtlasExploreResponse> {
        let limit = req.limit.unwrap_or(20);
        let depth = req.depth.unwrap_or(1);

        // Resolve seed IDs: region, node, or working memory
        let (region, seed_ids) = if let Some(region_id) = req.region_id {
            let region = self.get_atlas_region_by_id(region_id)?;
            let member_ids = self.get_region_member_ids(region_id)?;
            (region, member_ids)
        } else if let Some(node_id) = req.node_id {
            (None, vec![node_id])
        } else if req.from_working {
            let working_ids = self.working_memory_item_ids(
                req.project.as_deref(),
                req.namespace.as_deref(),
            )?;
            (None, working_ids)
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
                evidence: Vec::new(),
                truncated: false,
            });
        };

        // Load memory items for seed IDs, enriched with entity linkage
        let mut nodes = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for id in &seed_ids {
            if let Some(item) = self.get(*id)? {
                let entity = self.entity_for_item(item.id)?;
                if !passes_pivot_filters_with_entity(&item, entity.as_ref(), req) {
                    continue;
                }
                if seen.insert(item.id) {
                    let evidence_count = self.event_count_for_item(item.id)?;
                    nodes.push(item_to_atlas_node(
                        &item,
                        region.as_ref().map(|r| r.id),
                        0,
                        entity.as_ref().map(|e| e.id),
                        evidence_count,
                    ));
                }
            }
        }

        // Neighborhood expansion: entity links first, tag-overlap fallback
        let mut links = Vec::new();
        if depth > 0 {
            let mut found_via_entity = false;
            for seed_id in &seed_ids {
                let entity_links = self.get_entity_links_for_item(*seed_id)?;
                for el in entity_links {
                    let neighbor_id = if el.from_entity_id == *seed_id {
                        el.to_entity_id
                    } else {
                        el.from_entity_id
                    };
                    if let Some(neighbor_item) = self.get(neighbor_id)? {
                        if !passes_pivot_filters(&neighbor_item, req) {
                            continue;
                        }
                        if seen.insert(neighbor_item.id) {
                            found_via_entity = true;
                            let entity = self.entity_for_item(neighbor_item.id)?;
                            let evidence_count =
                                self.event_count_for_item(neighbor_item.id)?;
                            nodes.push(item_to_atlas_node(
                                &neighbor_item,
                                region.as_ref().map(|r| r.id),
                                1,
                                entity.as_ref().map(|e| e.id),
                                evidence_count,
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

            // Tag-overlap fallback: if no entity links found, find neighbors
            // sharing tags with seed items
            if !found_via_entity {
                let seed_tags: std::collections::HashSet<String> = nodes
                    .iter()
                    .flat_map(|n| n.tags.iter().cloned())
                    .collect();
                if !seed_tags.is_empty() {
                    let all_items = self.list()?;
                    let mut tag_neighbors: Vec<(usize, MemoryItem)> = all_items
                        .into_iter()
                        .filter(|item| item.status == MemoryStatus::Active)
                        .filter(|item| !seen.contains(&item.id))
                        .filter(|item| passes_pivot_filters(item, req))
                        .map(|item| {
                            let overlap = item
                                .tags
                                .iter()
                                .filter(|t| seed_tags.contains(t.as_str()))
                                .count();
                            (overlap, item)
                        })
                        .filter(|(overlap, _)| *overlap > 0)
                        .collect();
                    tag_neighbors.sort_by(|a, b| b.0.cmp(&a.0));

                    for (_, neighbor) in tag_neighbors.into_iter().take(limit / 2) {
                        if seen.insert(neighbor.id) {
                            let entity = self.entity_for_item(neighbor.id)?;
                            let evidence_count = self.event_count_for_item(neighbor.id)?;
                            // Link from first seed to neighbor
                            if let Some(first_seed) = seed_ids.first() {
                                links.push(AtlasLink {
                                    from_node_id: *first_seed,
                                    to_node_id: neighbor.id,
                                    link_kind: AtlasLinkKind::Semantic,
                                    weight: 0.4,
                                    label: Some("tag overlap".to_string()),
                                });
                            }
                            nodes.push(item_to_atlas_node(
                                &neighbor,
                                region.as_ref().map(|r| r.id),
                                1,
                                entity.as_ref().map(|e| e.id),
                                evidence_count,
                            ));
                        }
                    }
                }
            }
        }

        // Load persisted atlas links for seed nodes
        for seed_id in &seed_ids {
            let persisted = self.load_persisted_links_for_node(*seed_id)?;
            for pl in persisted {
                let neighbor_id = if pl.from_node_id == *seed_id {
                    pl.to_node_id
                } else {
                    pl.from_node_id
                };
                if let Some(item) = self.get(neighbor_id)? {
                    if !passes_pivot_filters(&item, req) {
                        continue;
                    }
                    if seen.insert(item.id) {
                        let entity = self.entity_for_item(item.id)?;
                        let evidence_count = self.event_count_for_item(item.id)?;
                        nodes.push(item_to_atlas_node(
                            &item,
                            region.as_ref().map(|r| r.id),
                            1,
                            entity.as_ref().map(|e| e.id),
                            evidence_count,
                        ));
                    }
                    links.push(pl);
                }
            }
        }

        // Correction-aware neighborhood: items linked via supersedes
        {
            let supersede_ids: Vec<Uuid> = seed_ids
                .iter()
                .filter_map(|id| self.get(*id).ok().flatten())
                .flat_map(|item| item.supersedes.clone())
                .collect();
            for sid in supersede_ids {
                if let Some(item) = self.get(sid)? {
                    if !passes_pivot_filters(&item, req) {
                        continue;
                    }
                    if seen.insert(item.id) {
                        let entity = self.entity_for_item(item.id)?;
                        let evidence_count = self.event_count_for_item(item.id)?;
                        nodes.push(item_to_atlas_node(
                            &item,
                            region.as_ref().map(|r| r.id),
                            1,
                            entity.as_ref().map(|e| e.id),
                            evidence_count,
                        ));
                    }
                    if let Some(first_seed) = seed_ids.first() {
                        links.push(AtlasLink {
                            from_node_id: *first_seed,
                            to_node_id: item.id,
                            link_kind: AtlasLinkKind::Corrective,
                            weight: 0.6,
                            label: Some("supersedes".to_string()),
                        });
                    }
                }
            }
        }

        // Time-based pivot
        if let Some(pivot_time) = req.pivot_time {
            nodes.retain(|node| {
                if let Ok(Some(item)) = self.get(node.memory_id) {
                    item.created_at <= pivot_time
                } else {
                    true
                }
            });
        }

        // Evidence loading: drill to raw events when requested
        let evidence = if req.include_evidence {
            let mut events = Vec::new();
            for node in &nodes {
                events.extend(self.events_for_item(node.memory_id)?);
            }
            events.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));
            events.truncate(50);
            events
        } else {
            Vec::new()
        };

        let trails = generate_trails(&nodes, &links);

        let truncated = nodes.len() > limit;
        nodes.truncate(limit);

        Ok(AtlasExploreResponse {
            region,
            nodes,
            links,
            trails,
            evidence,
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
            if let Some(p) = project
                && item.project.as_deref() != Some(p) {
                    continue;
                }
            if let Some(ns) = namespace
                && item.namespace.as_deref() != Some(ns) {
                    continue;
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
            if let Err(e) = self.upsert_atlas_region(&region) {
                eprintln!("warn: upsert_atlas_region: {e:#}");
            }
            let member_ids: Vec<Uuid> = members.iter().map(|m| m.id).collect();
            if let Err(e) = self.set_region_members(region_id, &member_ids) {
                eprintln!("warn: set_region_members: {e:#}");
            }
            regions.push(region);
        }

        regions.sort_by(|a, b| b.node_count.cmp(&a.node_count));
        Ok(regions)
    }

    pub(crate) fn atlas_expand(
        &self,
        req: &AtlasExpandRequest,
    ) -> anyhow::Result<AtlasExpandResponse> {
        let limit = req.limit.unwrap_or(10);
        let depth = req.depth.unwrap_or(1);
        let mut expanded_nodes = Vec::new();
        let mut links = Vec::new();
        let mut seen: std::collections::HashSet<Uuid> = req.memory_ids.iter().copied().collect();

        if depth > 0 {
            for seed_id in &req.memory_ids {
                let entity_links = self.get_entity_links_for_item(*seed_id)?;
                for el in entity_links {
                    let neighbor_id = if el.from_entity_id == *seed_id {
                        el.to_entity_id
                    } else {
                        el.from_entity_id
                    };
                    if let Some(item) = self.get(neighbor_id)? {
                        if item.status != MemoryStatus::Active {
                            continue;
                        }
                        if let Some(p) = &req.project
                            && item.project.as_deref() != Some(p) {
                                continue;
                            }
                        if seen.insert(item.id) {
                            let entity = self.entity_for_item(item.id)?;
                            let evidence_count = self.event_count_for_item(item.id)?;
                            expanded_nodes.push(item_to_atlas_node(
                                &item,
                                None,
                                1,
                                entity.as_ref().map(|e| e.id),
                                evidence_count,
                            ));
                        }
                        links.push(AtlasLink {
                            from_node_id: *seed_id,
                            to_node_id: item.id,
                            link_kind: entity_relation_to_atlas_link(el.relation_kind),
                            weight: el.confidence,
                            label: el.note.clone(),
                        });
                    }
                    if expanded_nodes.len() >= limit {
                        break;
                    }
                }
            }
        }

        expanded_nodes.truncate(limit);
        Ok(AtlasExpandResponse {
            seed_count: req.memory_ids.len(),
            expanded_nodes,
            links,
        })
    }

    pub(crate) fn save_atlas_trail(
        &self,
        req: &AtlasSaveTrailRequest,
    ) -> anyhow::Result<AtlasSaveTrailResponse> {
        let now = Utc::now();
        let trail = AtlasSavedTrail {
            id: deterministic_region_id(
                req.project.as_deref(),
                req.namespace.as_deref(),
                &req.name,
            ),
            name: req.name.clone(),
            project: req.project.clone(),
            namespace: req.namespace.clone(),
            region_id: req.region_id,
            node_ids: req.node_ids.clone(),
            created_at: now,
            updated_at: now,
        };
        let conn = self.connect()?;
        let payload = serde_json::to_string(&trail)?;
        conn.execute(
            r#"
            INSERT INTO atlas_trails (id, name, project, namespace, region_id, created_at, updated_at, payload_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
              name = excluded.name,
              region_id = excluded.region_id,
              updated_at = excluded.updated_at,
              payload_json = excluded.payload_json
            "#,
            params![
                trail.id.to_string(),
                trail.name,
                trail.project,
                trail.namespace,
                trail.region_id.map(|id| id.to_string()),
                trail.created_at.to_rfc3339(),
                trail.updated_at.to_rfc3339(),
                payload,
            ],
        )?;
        Ok(AtlasSaveTrailResponse { trail })
    }

    pub(crate) fn list_atlas_trails(
        &self,
        req: &AtlasListTrailsRequest,
    ) -> anyhow::Result<AtlasListTrailsResponse> {
        let conn = self.connect()?;
        let mut sql = String::from("SELECT payload_json FROM atlas_trails WHERE 1=1");
        let mut bind_values: Vec<String> = Vec::new();
        if let Some(project) = &req.project {
            sql.push_str(" AND project = ?");
            bind_values.push(project.clone());
        }
        if let Some(namespace) = &req.namespace {
            sql.push_str(" AND namespace = ?");
            bind_values.push(namespace.clone());
        }
        sql.push_str(" ORDER BY updated_at DESC");
        let limit = req.limit.unwrap_or(20);
        sql.push_str(&format!(" LIMIT {limit}"));

        let mut stmt = conn.prepare(&sql)?;
        let params: Vec<&dyn rusqlite::ToSql> = bind_values
            .iter()
            .map(|v| v as &dyn rusqlite::ToSql)
            .collect();
        let trails = stmt
            .query_map(params.as_slice(), |row| row.get::<_, String>(0))?
            .filter_map(|r| r.inspect_err(|e| eprintln!("warn: atlas trail row read: {e}")).ok())
            .filter_map(|json| serde_json::from_str::<AtlasSavedTrail>(&json).inspect_err(|e| eprintln!("warn: atlas trail json parse: {e}")).ok())
            .collect();
        Ok(AtlasListTrailsResponse { trails })
    }

    #[allow(dead_code)] // Reserved for Phase H atlas link persistence
    pub(crate) fn persist_atlas_link(&self, link: &AtlasLink) -> anyhow::Result<()> {
        let conn = self.connect()?;
        conn.execute(
            r#"
            INSERT INTO atlas_links (from_node_id, to_node_id, link_kind, weight, label, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(from_node_id, to_node_id, link_kind) DO UPDATE SET
              weight = excluded.weight,
              label = excluded.label
            "#,
            params![
                link.from_node_id.to_string(),
                link.to_node_id.to_string(),
                format!("{:?}", link.link_kind).to_lowercase(),
                link.weight,
                link.label,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub(crate) fn load_persisted_links_for_node(
        &self,
        node_id: Uuid,
    ) -> anyhow::Result<Vec<AtlasLink>> {
        let conn = self.connect()?;
        let id_str = node_id.to_string();
        let mut stmt = conn.prepare(
            "SELECT from_node_id, to_node_id, link_kind, weight, label FROM atlas_links WHERE from_node_id = ?1 OR to_node_id = ?1",
        )?;
        let links = stmt
            .query_map(params![id_str], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, f32>(3)?,
                    row.get::<_, Option<String>>(4)?,
                ))
            })?
            .filter_map(|r| r.inspect_err(|e| eprintln!("warn: atlas link row read: {e}")).ok())
            .filter_map(|(from, to, kind, weight, label)| {
                Some(AtlasLink {
                    from_node_id: from.parse().ok()?,
                    to_node_id: to.parse().ok()?,
                    link_kind: parse_link_kind(&kind)?,
                    weight,
                    label,
                })
            })
            .collect();
        Ok(links)
    }

    pub(crate) fn working_memory_item_ids(
        &self,
        project: Option<&str>,
        namespace: Option<&str>,
    ) -> anyhow::Result<Vec<Uuid>> {
        let items = self.list()?;
        let mut working: Vec<MemoryItem> = items
            .into_iter()
            .filter(|item| item.status == MemoryStatus::Active)
            .filter(|item| {
                item.kind == MemoryKind::Status
                    || item.kind == MemoryKind::LiveTruth
                    || item.kind == MemoryKind::Pattern
            })
            .filter(|item| {
                project.is_none() || item.project.as_deref() == project
            })
            .filter(|item| {
                namespace.is_none() || item.namespace.as_deref() == namespace
            })
            .collect();
        working.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        working.truncate(10);
        Ok(working.into_iter().map(|item| item.id).collect())
    }

    pub(crate) fn rename_atlas_region(
        &self,
        req: &AtlasRenameRegionRequest,
    ) -> anyhow::Result<AtlasRenameRegionResponse> {
        let mut region = self
            .get_atlas_region_by_id(req.region_id)?
            .ok_or_else(|| anyhow::anyhow!("region not found"))?;
        region.name = req.name.clone();
        region.description = req.description.clone();
        region.auto_generated = false;
        region.updated_at = Utc::now();
        self.upsert_atlas_region(&region)?;
        Ok(AtlasRenameRegionResponse { region })
    }

    pub(crate) fn events_for_item(
        &self,
        item_id: Uuid,
    ) -> anyhow::Result<Vec<MemoryEventRecord>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT payload_json FROM memory_events WHERE memory_item_id = ?1 ORDER BY recorded_at DESC LIMIT 10",
        )?;
        let events = stmt
            .query_map(params![item_id.to_string()], |row| {
                row.get::<_, String>(0)
            })?
            .filter_map(|r| r.inspect_err(|e| eprintln!("warn: event row read: {e}")).ok())
            .filter_map(|json| serde_json::from_str::<MemoryEventRecord>(&json).inspect_err(|e| eprintln!("warn: event json parse: {e}")).ok())
            .collect();
        Ok(events)
    }

    pub(crate) fn event_count_for_item(&self, item_id: Uuid) -> anyhow::Result<usize> {
        let conn = self.connect()?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM memory_events WHERE memory_item_id = ?1",
                params![item_id.to_string()],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(count as usize)
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
            .filter_map(|r| r.inspect_err(|e| eprintln!("warn: entity link row read: {e}")).ok())
            .filter_map(|json| serde_json::from_str(&json).inspect_err(|e| eprintln!("warn: entity link json parse: {e}")).ok())
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
        if let Some(filter) = lane_filter
            && lane != filter {
                return None;
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
    passes_pivot_filters_with_entity(item, None, req)
}

fn passes_pivot_filters_with_entity(
    item: &MemoryItem,
    entity: Option<&memd_schema::MemoryEntityRecord>,
    req: &AtlasExploreRequest,
) -> bool {
    if let Some(min_trust) = req.min_trust
        && item.confidence < min_trust {
            return false;
        }
    if let Some(min_salience) = req.min_salience {
        let salience = entity
            .map(|e| e.salience_score)
            .unwrap_or(item.confidence);
        if salience < min_salience {
            return false;
        }
    }
    if let Some(pivot_kind) = req.pivot_kind
        && item.kind != pivot_kind {
            return false;
        }
    if let Some(pivot_scope) = req.pivot_scope
        && item.scope != pivot_scope {
            return false;
        }
    if let Some(ref agent) = req.pivot_source_agent
        && item.source_agent.as_deref() != Some(agent) {
            return false;
        }
    if let Some(ref system) = req.pivot_source_system
        && item.source_system.as_deref() != Some(system) {
            return false;
        }
    if let Some(project) = &req.project
        && item.project.as_deref() != Some(project) {
            return false;
        }
    if let Some(namespace) = &req.namespace
        && item.namespace.as_deref() != Some(namespace) {
            return false;
        }
    true
}

fn item_to_atlas_node(
    item: &MemoryItem,
    region_id: Option<Uuid>,
    depth: usize,
    entity_id: Option<Uuid>,
    evidence_count: usize,
) -> AtlasNode {
    AtlasNode {
        id: item.id,
        region_id,
        memory_id: item.id,
        entity_id,
        label: compact_label(&item.content),
        kind: item.kind,
        stage: item.stage,
        lane: item_lane(item),
        confidence: item.confidence,
        salience: item.confidence,
        depth,
        evidence_count,
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

fn generate_trails(nodes: &[AtlasNode], links: &[AtlasLink]) -> Vec<memd_schema::AtlasTrail> {
    if nodes.len() < 2 {
        return Vec::new();
    }

    let mut trails = Vec::new();

    // Salience trail: highest confidence first — "most trusted path"
    let mut by_salience: Vec<&AtlasNode> = nodes.iter().collect();
    by_salience.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let salience_ids: Vec<Uuid> = by_salience.iter().map(|n| n.id).collect();
    let salience_links = trail_links_for_sequence(&salience_ids, links);
    trails.push(memd_schema::AtlasTrail {
        name: "salience".to_string(),
        nodes: salience_ids,
        links: salience_links,
    });

    // Depth trail: shallowest first — "zoom path" from region core to periphery
    let mut by_depth: Vec<&AtlasNode> = nodes.iter().collect();
    by_depth.sort_by_key(|n| n.depth);
    let depth_ids: Vec<Uuid> = by_depth.iter().map(|n| n.id).collect();
    if depth_ids != trails[0].nodes {
        let depth_links = trail_links_for_sequence(&depth_ids, links);
        trails.push(memd_schema::AtlasTrail {
            name: "zoom".to_string(),
            nodes: depth_ids,
            links: depth_links,
        });
    }

    trails
}

fn trail_links_for_sequence(node_ids: &[Uuid], all_links: &[AtlasLink]) -> Vec<AtlasLink> {
    let mut trail_links = Vec::new();
    for pair in node_ids.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        // Find existing link between these nodes
        if let Some(link) = all_links
            .iter()
            .find(|l| {
                (l.from_node_id == from && l.to_node_id == to)
                    || (l.from_node_id == to && l.to_node_id == from)
            })
        {
            trail_links.push(link.clone());
        } else {
            // Synthesize a temporal link for adjacency
            trail_links.push(AtlasLink {
                from_node_id: from,
                to_node_id: to,
                link_kind: AtlasLinkKind::Temporal,
                weight: 0.5,
                label: None,
            });
        }
    }
    trail_links
}

fn parse_link_kind(value: &str) -> Option<AtlasLinkKind> {
    match value {
        "temporal" => Some(AtlasLinkKind::Temporal),
        "causal" => Some(AtlasLinkKind::Causal),
        "procedural" => Some(AtlasLinkKind::Procedural),
        "semantic" => Some(AtlasLinkKind::Semantic),
        "corrective" => Some(AtlasLinkKind::Corrective),
        "ownership" => Some(AtlasLinkKind::Ownership),
        _ => None,
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
