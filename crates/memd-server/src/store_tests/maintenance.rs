use super::*;
    #[test]
    fn maintain_runtime_persists_report_receipt() {
        let dir = std::env::temp_dir().join(format!("runtime-maintain-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let store = SqliteStore::open(dir.join("state.sqlite")).expect("open sqlite store");

        let report = store
            .maintain_runtime(&MaintainReportRequest {
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                session: Some("session-a".to_string()),
                mode: "scan".to_string(),
                apply: false,
            })
            .expect("run maintain runtime");

        assert_eq!(report.mode, "scan");
        assert!(report.receipt_id.is_some());
        assert!(
            report
                .findings
                .iter()
                .any(|line| line.contains("memory maintain"))
        );

        let conn = store.connect().expect("connect sqlite store");
        let persisted: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM runtime_maintenance_reports",
                [],
                |row| row.get(0),
            )
            .expect("count persisted maintenance reports");
        assert_eq!(persisted, 1);

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }
