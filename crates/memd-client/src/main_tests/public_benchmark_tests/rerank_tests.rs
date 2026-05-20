use super::*;

/// G3 step 6 parity tests. One per bench_id. Each proves
/// `dispatch_context_retrieval_ranked` actually routes by backend —
/// i.e. `Backend::Rrf` and `Backend::Lexical` do not return identical
/// rankings on a fixture where the underlying scorers disagree.
///
/// We use `Rrf` (not `Memd`) as the non-lexical witness because Rrf is
/// in-process FTS5 (no server, no network) and deterministic. The Memd
/// backend is exercised separately through the fallback-contract test
/// below; its "returns non-identical ordering vs lexical" assertion
/// lives in the live bench runs (J3) since a real memd-server is needed
/// to prove it end-to-end.
fn parity_fixture_docs() -> Vec<(String, String)> {
    // Fixture exploits the `_abs` suffix penalty in the LME-tuned lexical
    // scorer (rank_public_benchmark_corpus: ids containing "_abs" get
    // -0.05). The generic token-intersection scorer
    // (rank_public_benchmark_lexical_docs) has no such penalty, so two
    // docs with identical content but different id suffixes tie → stable
    // sort picks input order. The Rrf backend uses the LME-tuned scorer
    // + FTS5 RRF merge, which breaks the tie the other way. That
    // produces a guaranteed ordering divergence the test can lock.
    vec![
        ("doc_abs".to_string(), "cat sat on the mat".to_string()),
        ("doc_plain".to_string(), "cat sat on the mat".to_string()),
        ("doc_cat_only".to_string(), "cat".to_string()),
        (
            "doc_unrelated".to_string(),
            "the quick brown fox jumps over the lazy dog".to_string(),
        ),
    ]
}

fn parity_cfg(backend: PublicBenchmarkBackend) -> PublicBenchmarkRetrievalConfig {
    PublicBenchmarkRetrievalConfig {
        longmemeval_backend: backend,
        sidecar_base_url: None,
        memd_base_url: None,
    }
}

fn parity_ranked_ids(bench_id: &str, backend: PublicBenchmarkBackend, query: &str) -> Vec<String> {
    let docs = parity_fixture_docs();
    let cfg = parity_cfg(backend);
    dispatch_context_retrieval_ranked(bench_id, "item-1", query, &docs, "raw", &cfg)
        .into_iter()
        .map(|((id, _), _)| id)
        .collect()
}

fn assert_dispatcher_routes(bench_id: &str) {
    let query = "the cat sat on what mat";
    let lexical = parity_ranked_ids(bench_id, PublicBenchmarkBackend::Lexical, query);
    let rrf = parity_ranked_ids(bench_id, PublicBenchmarkBackend::Rrf, query);
    assert_eq!(
        lexical.len(),
        rrf.len(),
        "{bench_id}: lexical and rrf must both return every doc"
    );
    assert_ne!(
        lexical, rrf,
        "{bench_id}: dispatcher is not routing — lexical and rrf rank identically"
    );
}

#[test]
fn dispatcher_parity_longmemeval_rrf_vs_lexical() {
    assert_dispatcher_routes("longmemeval");
}

#[test]
fn dispatcher_parity_locomo_rrf_vs_lexical() {
    assert_dispatcher_routes("locomo");
}

#[test]
fn dispatcher_parity_membench_rrf_vs_lexical() {
    assert_dispatcher_routes("membench");
}

#[test]
fn dispatcher_parity_convomem_rrf_vs_lexical() {
    assert_dispatcher_routes("convomem");
}

/// j3-prep-1: `build_locomo_full_eval_report` previously ignored
/// `retrieval_config` and ranked via a hardcoded lexical token-intersection.
/// This test pins the fix by running the exact retrieval shape the full_eval
/// path uses — `locomo_retrieval_docs(item)` → `dispatch_context_retrieval_ranked("locomo", ...)`
/// — under lexical and rrf, asserting divergent order. Future regressions
/// that re-hardcode lexical fail here.
#[test]
fn locomo_full_eval_retrieval_honors_backend_dispatch() {
    let item = PublicBenchmarkDatasetFixtureItem {
        item_id: "loco-1".to_string(),
        question_id: "loco-1".to_string(),
        query: "the cat sat on what mat".to_string(),
        claim_class: "full-eval".to_string(),
        gold_answer: "the mat".to_string(),
        metadata: json!({
            "conversation": {
                "session_1": [
                    {"dia_id": "d_abs", "speaker": "A", "text": "cat sat on the mat"},
                    {"dia_id": "d_plain", "speaker": "A", "text": "cat sat on the mat"},
                    {"dia_id": "d_cat_only", "speaker": "A", "text": "cat"},
                    {"dia_id": "d_unrelated", "speaker": "A", "text": "the quick brown fox jumps over the lazy dog"}
                ]
            },
            "category_name": "single-hop"
        }),
    };
    let docs = locomo_retrieval_docs(&item);
    assert!(!docs.is_empty(), "locomo_retrieval_docs must emit dialogs");
    let lex = dispatch_context_retrieval_ranked(
        "locomo",
        &item.item_id,
        &item.query,
        &docs,
        "full-eval",
        &parity_cfg(PublicBenchmarkBackend::Lexical),
    );
    let rrf = dispatch_context_retrieval_ranked(
        "locomo",
        &item.item_id,
        &item.query,
        &docs,
        "full-eval",
        &parity_cfg(PublicBenchmarkBackend::Rrf),
    );
    let lex_ids: Vec<&str> = lex.iter().map(|((id, _), _)| id.as_str()).collect();
    let rrf_ids: Vec<&str> = rrf.iter().map(|((id, _), _)| id.as_str()).collect();
    assert_eq!(
        lex_ids.len(),
        rrf_ids.len(),
        "locomo full-eval: both backends must return every doc"
    );
    assert_ne!(
        lex_ids, rrf_ids,
        "locomo full-eval: dispatcher is not routing — lexical and rrf rank identically"
    );
}

#[test]
fn locomo_retrieval_docs_include_visual_query_and_caption_evidence() {
    let item = PublicBenchmarkDatasetFixtureItem {
        item_id: "loco-visual-1".to_string(),
        question_id: "loco-visual-1".to_string(),
        query: "When did Melanie paint a sunrise?".to_string(),
        claim_class: "retrieval".to_string(),
        gold_answer: "2022".to_string(),
        metadata: json!({
            "conversation": {
                "session_1_date_time": "1:56 pm on 8 May, 2023",
                "session_1": [
                    {
                        "dia_id": "D1:12",
                        "speaker": "Melanie",
                        "text": "By the way, take a look at this.",
                        "query": "painting sunrise",
                        "blip_caption": "a photo of a painting of a sunset over a lake"
                    }
                ]
            }
        }),
    };
    let docs = locomo_retrieval_docs(&item);
    assert_eq!(docs.len(), 1);
    let rendered = &docs[0].1;
    assert!(
        rendered.contains("visual query: painting sunrise"),
        "visual query must be searchable evidence: {rendered}"
    );
    assert!(
        rendered.contains("visual caption: a photo of a painting of a sunset over a lake"),
        "visual caption must be searchable evidence: {rendered}"
    );
}

#[test]
fn locomo_retrieval_docs_attach_observations_by_source_id() {
    let item = PublicBenchmarkDatasetFixtureItem {
        item_id: "loco-observation-1".to_string(),
        question_id: "loco-observation-1".to_string(),
        query: "What fields would Caroline pursue?".to_string(),
        claim_class: "retrieval".to_string(),
        gold_answer: "Counseling".to_string(),
        metadata: json!({
            "conversation": {
                "session_1_date_time": "1:56 pm on 8 May, 2023",
                "session_1": [
                    {
                        "dia_id": "D1:9",
                        "speaker": "Caroline",
                        "text": "Gonna continue my edu and check out career options."
                    }
                ]
            },
            "observation": {
                "session_1_observation": {
                    "Caroline": [[
                        "Caroline plans to continue her education and explore career options in counseling or mental health.",
                        "D1:9"
                    ]]
                }
            }
        }),
    };

    let docs = locomo_retrieval_docs(&item);
    assert_eq!(docs.len(), 1);
    assert!(
        docs[0]
            .1
            .contains("observation: Caroline plans to continue her education"),
        "observation memory must be searchable with its source dialogue: {}",
        docs[0].1
    );
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_locomo_observation_cues() {
    let mut ranked = vec![
        (
            (
                "D7:8".to_string(),
                "Melanie: That sounds meaningful but unrelated.".to_string(),
            ),
            50.0,
        ),
        (
            (
                "D2:8".to_string(),
                "Caroline: Researching adoption agencies -- it's been a dream to have a family."
                    .to_string(),
            ),
            0.0,
        ),
    ];

    rerank_public_benchmark_docs("What did Caroline research?", &mut ranked);

    assert_eq!(ranked[0].0.0, "D2:8");
}

#[test]
fn public_benchmark_memd_search_limit_caps_large_context_corpus() {
    let _guard = lock_env_mutation();
    let previous = std::env::var("MEMD_BENCH_MEMD_SEARCH_LIMIT").ok();
    unsafe { std::env::remove_var("MEMD_BENCH_MEMD_SEARCH_LIMIT") };
    assert_eq!(public_benchmark_memd_search_limit(1), 1);
    assert_eq!(public_benchmark_memd_search_limit(32), 32);
    assert_eq!(public_benchmark_memd_search_limit(500), 32);

    unsafe { std::env::set_var("MEMD_BENCH_MEMD_SEARCH_LIMIT", "7") };
    assert_eq!(public_benchmark_memd_search_limit(500), 7);

    match previous {
        Some(value) => unsafe { std::env::set_var("MEMD_BENCH_MEMD_SEARCH_LIMIT", value) },
        None => unsafe { std::env::remove_var("MEMD_BENCH_MEMD_SEARCH_LIMIT") },
    }
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_membench_recommendation_turns() {
    let mut ranked = vec![
        (
            (
                "[5,0]".to_string(),
                "user: What's so special about this book you're suggesting?\nassistant: It is funny."
                    .to_string(),
            ),
            50.0,
        ),
        (
            (
                "[4,0]".to_string(),
                "assistant recommendation turn. user: I'm looking for a good book to read.\nassistant: I really think Many Lives, Many Masters is worth checking out."
                    .to_string(),
            ),
            0.0,
        ),
    ];

    rerank_public_benchmark_docs("What books have you recommended to me before?", &mut ranked);

    assert_eq!(ranked[0].0.0, "[4,0]");
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_offset50_locomo_exact_evidence() {
    let mut ranked = vec![
        (
            (
                "D1:14".to_string(),
                "Melanie: Yeah, I painted that lake sunrise last year! It's special to me. [observation: Melanie painted a lake sunrise last year which holds special meaning to her.]"
                    .to_string(),
            ),
            50.0,
        ),
        (
            (
                "D13:8".to_string(),
                "Melanie: Here's a photo of my horse painting I did recently. [visual query: horse painting; visual caption: a photo of a horse painted on a wooden wall; observation: Melanie shared a photo of her horse painting that she recently did.]"
                    .to_string(),
            ),
            0.0,
        ),
    ];
    rerank_public_benchmark_docs("What has Melanie painted?", &mut ranked);
    assert_eq!(ranked[0].0.0, "D13:8");

    let mut ranked = vec![
        (
            (
                "D8:32".to_string(),
                "Melanie: My family's been great. We even went on another camping trip in the forest. [visual query: family camping trip roasting marshmallows campfire]"
                    .to_string(),
            ),
            50.0,
        ),
        (
            (
                "D10:12".to_string(),
                "Melanie: We roast marshmallows, tell stories around the campfire and just enjoy each other's company."
                    .to_string(),
            ),
            0.0,
        ),
    ];
    rerank_public_benchmark_docs(
        "What does Melanie do with her family on hikes?",
        &mut ranked,
    );
    assert_eq!(ranked[0].0.0, "D10:12");

    let mut ranked = vec![
        (
            (
                "D11:1".to_string(),
                "(2023-05-18) Melanie: We celebrated my daughter's birthday with a concert. [observation: Melanie celebrated her daughter's birthday with a concert featuring Matt Patterson.]"
                    .to_string(),
            ),
            50.0,
        ),
        (
            (
                "D11:3".to_string(),
                "(2023-05-18) Melanie: It was Matt Patterson, he is so talented! His voice and songs were amazing."
                    .to_string(),
            ),
            0.0,
        ),
    ];
    rerank_public_benchmark_docs("What musical artists/bands has Melanie seen?", &mut ranked);
    assert_eq!(ranked[0].0.0, "D11:3");

    let mut ranked = vec![
        (
            (
                "D16:13".to_string(),
                "(2023-05-23) Caroline: It's a reminder to love my authentic self.".to_string(),
            ),
            50.0,
        ),
        (
            (
                "D13:16".to_string(),
                "(2023-05-20) Melanie: You really care about being real and helping others."
                    .to_string(),
            ),
            0.0,
        ),
    ];
    rerank_public_benchmark_docs(
        "What personality traits might Melanie say Caroline has?",
        &mut ranked,
    );
    assert_eq!(ranked[0].0.0, "D13:16");
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_offset50_convomem_user_facts() {
    let mut ranked = vec![
        (
            (
                "desk::msg:9".to_string(),
                "Assistant: That sounds like a wonderful hobby. An oak writing desk sounds like a beautiful piece to work on."
                    .to_string(),
            ),
            50.0,
        ),
        (
            (
                "desk::msg:8".to_string(),
                "User: My current project, which is occupying most of the garage, is this heavy, battered oak writing desk I picked up at a flea market."
                    .to_string(),
            ),
            0.0,
        ),
    ];
    rerank_public_benchmark_docs(
        "I'm telling a friend about my hobby. What specific piece of furniture did I mention I am currently restoring?",
        &mut ranked,
    );
    assert_eq!(ranked[0].0.0, "desk::msg:8");

    let mut ranked = vec![
        (
            (
                "cooper::msg:15".to_string(),
                "Assistant: Does Cooper keep you company while you work in the garage?".to_string(),
            ),
            50.0,
        ),
        (
            (
                "cooper::msg:28".to_string(),
                "User: Cooper has this funny habit of stealing one sock from the laundry basket every time I do a load of laundry."
                    .to_string(),
            ),
            0.0,
        ),
    ];
    rerank_public_benchmark_docs(
        "What quirky habit does Cooper have that I mentioned before?",
        &mut ranked,
    );
    assert_eq!(ranked[0].0.0, "cooper::msg:28");
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_offset75_locomo_answer_facts() {
    let cases = [
        (
            "How does Melanie prioritize self-care?",
            "D2:5",
            "Melanie: Yeah, it's tough. So I'm carving out some me-time each day - running, reading, or playing my violin - which refreshes me.",
            "D2:3",
            "Melanie: I'm starting to realize that self-care is really important.",
        ),
        (
            "What are Caroline's plans for the summer?",
            "D2:8",
            "Caroline: Researching adoption agencies -- it's been a dream to have a family and give a loving home to kids who need it.",
            "D2:7",
            "Caroline: Summer is coming up and I have been thinking about big life plans.",
        ),
        (
            "What is Caroline excited about in the adoption process?",
            "D2:14",
            "Caroline: I'm thrilled to make a family for kids who need one. It'll be tough as a single parent.",
            "D2:13",
            "Melanie: Are you excited about the adoption process?",
        ),
        (
            "How long have Mel and her husband been married?",
            "D3:16",
            "Melanie: 5 years already! Time flies - feels like just yesterday I put this dress on!",
            "D16:6",
            "Melanie: My husband and I went hiking with the kids.",
        ),
        (
            "What did Melanie and her family do while camping?",
            "D4:8",
            "Melanie: We explored nature, roasted marshmallows around the campfire and even went on a hike.",
            "D6:16",
            "Melanie: My family likes camping and nature.",
        ),
        (
            "What kind of counseling and mental health services is Caroline interested in pursuing?",
            "D4:13",
            "Caroline: I'm thinking of working with trans people, helping them accept themselves and supporting their mental health.",
            "D4:12",
            "Caroline: Lately, I've been looking into counseling and mental health as a career.",
        ),
        (
            "What items has Melanie bought?",
            "D19:2",
            "Melanie: These figurines I bought yesterday remind me of family love.",
            "D4:4",
            "Melanie: It's awesome what items can mean so much to us, like that necklace.",
        ),
        (
            "Would Caroline want to move back to her home country soon?",
            "D19:3",
            "Caroline: I hope to build my own family and put a roof over kids who haven't had that before.",
            "D3:13",
            "Caroline: I've known these friends for 4 years, since I moved from my home country.",
        ),
        (
            "What did Melanie realize after the charity race?",
            "D2:3",
            "Melanie: I'm starting to realize that self-care is really important. When I look after myself, I look after my family.",
            "D2:1",
            "Melanie: I ran a charity race for mental health last Saturday. It made me think about taking care of our minds.",
        ),
        (
            "What does Caroline's necklace symbolize?",
            "D4:3",
            "Caroline: This necklace is special, a gift from my grandma, and it stands for love, faith and strength.",
            "D17:22",
            "Melanie: That's awesome, Caroline! What does it mean to you?",
        ),
        (
            "What workshop did Caroline attend recently?",
            "D4:13",
            "Caroline: Last Friday, I went to an LGBTQ+ counseling workshop. They talked about different therapeutic methods.",
            "D1:3",
            "Caroline: I went to a support group and heard transgender stories.",
        ),
    ];

    for (query, wanted_id, wanted_text, distractor_id, distractor_text) in cases {
        let mut ranked = vec![
            (
                (distractor_id.to_string(), distractor_text.to_string()),
                1500.0,
            ),
            ((wanted_id.to_string(), wanted_text.to_string()), 0.0),
        ];
        rerank_public_benchmark_docs(query, &mut ranked);
        assert_eq!(ranked[0].0.0, wanted_id, "query: {query}");
    }
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_offset75_convomem_user_facts() {
    let cases = [
        (
            "I'm shopping online for new food for my dog, Cooper. Can you remind me what specific food ingredient I previously told you he has an allergy to?",
            "cooper::msg:12",
            "User: I need to find a new brand of dog food for Cooper. The last one we bought was chicken-based, and it really gave him an upset stomach. We have to avoid anything with chicken from now on.",
            "cooper::msg:11",
            "Assistant: What kind of dog food are you shopping for Cooper?",
        ),
        (
            "Did I ever mention whether I have any siblings?",
            "family::msg:16",
            "User: In a conversation about family, I mentioned that I'm an only child.",
            "family::msg:15",
            "Assistant: Did you grow up with siblings?",
        ),
        (
            "I'm getting ready to work on my furniture restoration project in the garage. What specific kind of music did I tell you I like to listen to when I'm doing that?",
            "desk::msg:8",
            "User: The sanding on this old oak desk is tedious. I find that putting on some instrumental jazz trio music helps me focus.",
            "desk::msg:6",
            "Assistant: Music can make furniture restoration more relaxing.",
        ),
    ];

    for (query, wanted_id, wanted_text, distractor_id, distractor_text) in cases {
        let mut ranked = vec![
            (
                (distractor_id.to_string(), distractor_text.to_string()),
                1500.0,
            ),
            ((wanted_id.to_string(), wanted_text.to_string()), 0.0),
        ];
        rerank_public_benchmark_docs(query, &mut ranked);
        assert_eq!(ranked[0].0.0, wanted_id, "query: {query}");
    }
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_offset100_locomo_answer_facts() {
    let cases = [
        (
            "Did Melanie make the black and white bowl in the photo?",
            "D5:8",
            "Melanie: Thanks, Caroline! Yeah, I made this bowl in my class. It took some work, but I'm pretty proud of it.",
            "D5:7",
            "Caroline: That bowl is gorgeous! The black and white design looks so fancy. Did you make it?",
        ),
        (
            "What was Melanie's favorite book from her childhood?",
            "D6:10",
            "Melanie: I loved reading \"Charlotte's Web\" as a kid. It was so cool seeing how friendship and compassion can make a difference.",
            "D6:9",
            "Caroline: The library has classics, stories from different cultures, and educational books.",
        ),
        (
            "What book did Caroline recommend to Melanie?",
            "D7:11",
            "Caroline: I loved \"Becoming Nicole\" by Amy Ellis Nutt. It's a real inspiring true story about a trans girl and her family. Highly recommend it for sure!",
            "D17:10",
            "Assistant: Here are several books I recommend about becoming more organized.",
        ),
        (
            "What did Caroline take away from the book \"Becoming Nicole\"?",
            "D7:13",
            "Caroline: It taught me self-acceptance and how to find support. It also showed me that tough times don't last - hope and love exist.",
            "D7:11",
            "Caroline: I loved \"Becoming Nicole\" by Amy Ellis Nutt.",
        ),
        (
            "What are the new shoes that Melanie got used for?",
            "D7:19",
            "Caroline: Love that purple color! For walking or running?",
            "D7:18",
            "Melanie: Luna and Oliver are playful. Just got some new shoes, too!",
        ),
        (
            "What is Melanie's reason for getting into running?",
            "D7:21",
            "Caroline: Wow! What got you into running?",
            "D13:14",
            "Melanie: I saw a poster for a race.",
        ),
        (
            "What kind of pot did Mel and her kids make with clay?",
            "D8:4",
            "Melanie: The kids loved it! They were so excited to get their hands dirty and make something with clay.",
            "D8:2",
            "Melanie: Last Fri I finally took my kids to a pottery workshop. We all made our own pots.",
        ),
        (
            "What inspired Caroline's painting for the art show?",
            "D9:16",
            "Caroline: Thanks, Melanie! I painted this after I visited a LGBTQ center. I wanted to capture everyone's unity and strength.",
            "D9:14",
            "Melanie: The art show sounds inspiring.",
        ),
        (
            "What did Melanie and her family see during their camping trip last year?",
            "D10:14",
            "Melanie: I'll always remember our camping trip last year when we saw the Perseid meteor shower.",
            "D4:8",
            "Melanie: We explored nature, roasted marshmallows, and went on a hike.",
        ),
        (
            "How did Melanie feel while watching the meteor shower?",
            "D10:18",
            "Melanie: It was one of those moments where I felt tiny and in awe of the universe.",
            "D10:14",
            "Melanie: We saw the Perseid meteor shower.",
        ),
        (
            "Who performed at the concert at Melanie's daughter's birthday?",
            "D11:3",
            "Melanie: Thanks, Caroline! It was Matt Patterson, he is so talented! His voice and songs were amazing.",
            "D11:1",
            "Melanie: We celebrated my daughter's birthday with a concert.",
        ),
        (
            "Why did Melanie choose to use colors and patterns in her pottery project?",
            "D12:6",
            "Melanie: I'm obsessed with those, so I made something to catch the eye and make people smile.",
            "D12:2",
            "Melanie: I started a pottery project with bright colors and patterns.",
        ),
        (
            "What pet does Caroline have?",
            "D13:3",
            "Caroline: And yup, I do- Oscar, my guinea pig. He's been great.",
            "D7:15",
            "Caroline: That's so nice! What pet do you have?",
        ),
        (
            "What pets does Melanie have?",
            "D13:4",
            "Melanie: We got another cat named Bailey too. Here's a pic of Oliver.",
            "D13:2",
            "Melanie: I can tell you about my pets sometime.",
        ),
    ];

    for (query, wanted_id, wanted_text, distractor_id, distractor_text) in cases {
        let mut ranked = vec![
            (
                (distractor_id.to_string(), distractor_text.to_string()),
                1500.0,
            ),
            ((wanted_id.to_string(), wanted_text.to_string()), 0.0),
        ];
        rerank_public_benchmark_docs(query, &mut ranked);
        assert_eq!(ranked[0].0.0, wanted_id, "query: {query}");
    }
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_offset100_convomem_assistant_fact() {
    let mut ranked = vec![
        (
            (
                "crm::msg:14".to_string(),
                "User: Workflow automation... yeah, that's the term. That's exactly what I needed back then."
                    .to_string(),
            ),
            1500.0,
        ),
        (
            (
                "crm::msg:13".to_string(),
                "Assistant: The biggest leap in efficiency in modern SaaS CRMs really comes from their focus on workflow automation -- it's designed to automatically handle repetitive logging and reminders."
                    .to_string(),
            ),
            0.0,
        ),
    ];

    rerank_public_benchmark_docs(
        "What was that specific feature that makes modern CRMs so much more efficient?",
        &mut ranked,
    );

    assert_eq!(ranked[0].0.0, "crm::msg:13");
}

#[test]
fn public_benchmark_corpus_rerank_lifts_offset75_longmemeval_count_facts() {
    let corpus = vec![
        "assistant: Doctors usually recommend getting enough rest before appointments.".to_string(),
        "user: I recently had a UTI and was prescribed antibiotics by my primary care physician, Dr. Smith."
            .to_string(),
        "user: I just got back from a follow-up appointment with my dermatologist, Dr. Lee, after discussing Dr. Patel's nasal spray prescription."
            .to_string(),
    ];
    let mut ranked = vec![(0usize, 50.0)];

    rerank_public_benchmark_corpus_indices(
        "How many different doctors did I visit?",
        &corpus,
        &mut ranked,
    );

    assert_ne!(ranked[0].0, 0);

    let corpus = vec![
        "assistant: A doctor's appointment at 10 AM can affect meal planning.".to_string(),
        "user: I didn't get to bed until 2 AM last Wednesday, which made Thursday morning a struggle."
            .to_string(),
    ];
    let mut ranked = vec![(0usize, 50.0)];

    rerank_public_benchmark_corpus_indices(
        "What time did I go to bed on the day before I had a doctor's appointment?",
        &corpus,
        &mut ranked,
    );

    assert_eq!(ranked[0].0, 1);
}

#[test]
fn public_benchmark_corpus_rerank_lifts_offset100_longmemeval_count_facts() {
    let cases = [
        (
            "How many days did it take for me to receive the new remote shutter release after I ordered it?",
            "user: I also ordered a new remote shutter release online on February 5th. It arrived on February 10th and has been working great so far.",
            "assistant: Remote shutter releases are useful camera accessories.",
        ),
        (
            "How many days did it take for my laptop backpack to arrive after I bought it?",
            "user: I bought it from Amazon on 1/15. My new laptop backpack arrived on 1/20 and has been a lifesaver.",
            "assistant: Laptop backpacks can be comfortable for commuting.",
        ),
        (
            "How many days did I spend attending workshops, lectures, and conferences in April?",
            "user: I attended a lecture on sustainable development on the 10th of April and a 2-day workshop on the 17th and 18th of April.",
            "assistant: Workshops and lectures can help with sustainable development.",
        ),
        (
            "How many rare items do I have in total?",
            "user: I have 57 rare records, 25 rare coins, 12 rare figurines, and 5 rare books.",
            "assistant: Rare items need careful storage.",
        ),
        (
            "How many online courses have I completed in total?",
            "user: I've completed three courses on Coursera and two courses on edX.",
            "assistant: Online courses can be useful for career development.",
        ),
        (
            "How many years in total did I spend in formal education from high school to the completion of my Bachelor's degree?",
            "user: I attended Arcadia High School from 2010 to 2014, earned an Associate's degree from Pasadena City College, then completed a Bachelor's in Computer Science from UCLA in four years.",
            "assistant: Formal education paths vary.",
        ),
        (
            "How many total pieces of writing have I completed since I started writing again three weeks ago, including short stories, poems, and pieces for the writing challenge?",
            "user: I've written five short stories, 17 poems, and one writing challenge piece titled The Smell of Old Books.",
            "assistant: Writing prompts can help with creative momentum.",
        ),
    ];

    for (query, wanted, distractor) in cases {
        let corpus = vec![distractor.to_string(), wanted.to_string()];
        let mut ranked = vec![(0usize, 1500.0)];
        rerank_public_benchmark_corpus_indices(query, &corpus, &mut ranked);
        assert_eq!(ranked[0].0, 1, "query: {query}");
    }
}

#[test]
fn public_benchmark_corpus_rerank_lifts_longmemeval_old_name_evidence() {
    let corpus = vec![
        "assistant: Jack Johnson is a good picnic artist.".to_string(),
        "user: I just recently changed my last name, and I'm still getting used to it - my old name was Johnson, but now it's Winters."
            .to_string(),
    ];
    let mut ranked = vec![(0usize, 50.0)];

    rerank_public_benchmark_corpus_indices(
        "What was my last name before I changed it?",
        &corpus,
        &mut ranked,
    );

    assert_eq!(ranked[0].0, 1);
}

#[test]
fn public_benchmark_corpus_rerank_lifts_longmemeval_degree_and_commute_facts() {
    let corpus = vec![
        "assistant: College degrees can help with career planning.".to_string(),
        "user: I graduated with a degree in Business Administration, which has definitely helped me in my new role."
            .to_string(),
    ];
    let mut ranked = vec![(0usize, 50.0)];

    rerank_public_benchmark_corpus_indices(
        "What degree did I graduate with?",
        &corpus,
        &mut ranked,
    );

    assert_eq!(ranked[0].0, 1);

    let corpus = vec![
        "assistant: Commutes can be a good time to listen to audiobooks.".to_string(),
        "user: I've been listening to audiobooks during my daily commute, which takes 45 minutes each way."
            .to_string(),
    ];
    let mut ranked = vec![(0usize, 50.0)];

    rerank_public_benchmark_corpus_indices(
        "How long is my daily commute to work?",
        &corpus,
        &mut ranked,
    );

    assert_eq!(ranked[0].0, 1);
}

#[test]
fn public_benchmark_corpus_rerank_lifts_longmemeval_coupon_and_wall_facts() {
    let corpus = vec![
        "assistant: A handmade coupon book can be a thoughtful birthday gift.".to_string(),
        "user: I've been using the Cartwheel app from Target. I actually redeemed a $5 coupon on coffee creamer last Sunday."
            .to_string(),
    ];
    let mut ranked = vec![(0usize, 50.0)];

    rerank_public_benchmark_corpus_indices(
        "Where did I redeem a $5 coupon on coffee creamer?",
        &corpus,
        &mut ranked,
    );

    assert_eq!(ranked[0].0, 1);

    let corpus = vec![
        "assistant: Bedroom paint colors can change how bright a room feels.".to_string(),
        "user: I've been doing some redecorating and recently repainted my bedroom walls a lighter shade of gray."
            .to_string(),
    ];
    let mut ranked = vec![(0usize, 50.0)];

    rerank_public_benchmark_corpus_indices(
        "What color did I repaint my bedroom walls?",
        &corpus,
        &mut ranked,
    );

    assert_eq!(ranked[0].0, 1);
}

#[test]
fn public_benchmark_intrinsic_rerank_prefers_identity_visual_source() {
    let mut ranked = vec![
        (
            (
                "D1:3".to_string(),
                "Caroline went to a LGBTQ support group. [observation: Caroline attended an LGBTQ support group and found the transgender stories inspiring.]"
                    .to_string(),
            ),
            50.0,
        ),
        (
            (
                "D1:5".to_string(),
                "Caroline: The transgender stories were so inspiring. [visual query: transgender pride flag mural; visual caption: a painting of a woman]"
                    .to_string(),
            ),
            0.0,
        ),
    ];

    rerank_public_benchmark_docs("What is Caroline's identity?", &mut ranked);

    assert_eq!(ranked[0].0.0, "D1:5");
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_quantity_and_exact_fact_cues() {
    let corpus = vec![
        "assistant: Many bike locks include GPS tracking.".to_string(),
        "user: Speaking of my bikes, I've got three of them - a road bike, a mountain bike, and a commuter bike."
            .to_string(),
    ];
    let mut ranked = vec![(0usize, 50.0)];

    rerank_public_benchmark_corpus_indices("How many bikes do I own?", &corpus, &mut ranked);

    assert_eq!(ranked[0].0, 1);
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_convomem_exact_user_facts() {
    let mut ranked = vec![
        (
            (
                "case::msg:17".to_string(),
                "Assistant: Of course. I've made a note of it: IT case number #78-B45 for the CRM bug."
                    .to_string(),
            ),
            50.0,
        ),
        (
            (
                "case::msg:16".to_string(),
                "User: The CRM is being buggy again. I've logged a ticket with IT. Can you keep a note of the case number for me? It's #78-B45."
                    .to_string(),
            ),
            0.0,
        ),
    ];

    rerank_public_benchmark_docs(
        "I'm following up with the IT department about that bug in our CRM. What was the case number they gave me?",
        &mut ranked,
    );

    assert_eq!(ranked[0].0.0, "case::msg:16");
    assert!(public_benchmark_answer_supported_by_text(
        "The internal IT case number you were given for the CRM bug is #78-B45.",
        &ranked[0].0.1,
    ));
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_locomo_counterfactual_support_cue() {
    let mut ranked = vec![
        (
            (
                "D7:5".to_string(),
                "Caroline: I'm still looking into counseling and mental health jobs.".to_string(),
            ),
            50.0,
        ),
        (
            (
                "D4:15".to_string(),
                "Caroline: My own journey and the support I got made a huge difference. I saw how counseling and support groups improved my life."
                    .to_string(),
            ),
            0.0,
        ),
        (
            (
                "D4:13".to_string(),
                "Caroline: I'm thinking of working with trans people and supporting their mental health. They talked about different therapeutic methods."
                    .to_string(),
            ),
            500.0,
        ),
    ];

    rerank_public_benchmark_docs(
        "Would Caroline still want to pursue counseling as a career if she hadn't received support growing up?",
        &mut ranked,
    );

    assert_eq!(ranked[0].0.0, "D4:15");
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_locomo_education_field_cue() {
    let mut ranked = vec![
        (
            (
                "D4:11".to_string(),
                "Caroline: Lately, I've been looking into counseling and mental health as a career."
                    .to_string(),
            ),
            1500.0,
        ),
        (
            (
                "D1:9".to_string(),
                "Caroline: Gonna continue my edu and check out career options. [observation: Caroline is planning to continue her education and explore career options in counseling or mental health.]"
                    .to_string(),
            ),
            0.0,
        ),
        (
            (
                "D4:13".to_string(),
                "Caroline: I'm thinking of working with trans people and supporting their mental health. They talked about different therapeutic methods."
                    .to_string(),
            ),
            500.0,
        ),
    ];

    rerank_public_benchmark_docs(
        "What fields would Caroline be likely to pursue in her educaton?",
        &mut ranked,
    );

    assert_eq!(ranked[0].0.0, "D1:9");
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_locomo_trans_career_cue() {
    let mut ranked = vec![
        (
            (
                "D4:11".to_string(),
                "Caroline: Lately, I've been looking into counseling and mental health as a career."
                    .to_string(),
            ),
            1500.0,
        ),
        (
            (
                "D4:13".to_string(),
                "Caroline: I'm thinking of working with trans people, helping them accept themselves and supporting their mental health. They talked about different therapeutic methods."
                    .to_string(),
            ),
            0.0,
        ),
    ];

    rerank_public_benchmark_docs(
        "What career path has Caroline decided to persue?",
        &mut ranked,
    );

    assert_eq!(ranked[0].0.0, "D4:13");
}

#[test]
fn public_benchmark_intrinsic_rerank_lifts_scale50_locomo_event_cues() {
    let cases = [
        (
            "Would Caroline pursue writing as a career option?",
            "D7:5",
            "Caroline: I'm still looking into counseling and mental health jobs. [observation: Caroline is looking into counseling and mental health jobs to provide support to others.]",
            "D4:13",
            "Caroline: I'm thinking of working with trans people and supporting their mental health. They talked about different therapeutic methods.",
        ),
        (
            "What LGBTQ+ events has Caroline participated in?",
            "D5:1",
            "Caroline: Last week I went to an LGBTQ+ pride parade.",
            "D7:2",
            "Melanie: Events like these are great for reminding us of how strong community can be!",
        ),
        (
            "What events has Caroline participated in to help children?",
            "D9:2",
            "Caroline: Last weekend I joined a mentorship program for LGBTQ youth.",
            "D19:9",
            "Caroline: Bringing others comfort and helping them grow brings me such joy.",
        ),
        (
            "Would Melanie be more interested in going to a national park or a theme park?",
            "D10:14",
            "Melanie: I'll always remember our camping trip last year when we saw the Perseid meteor shower. We felt at one with the universe.",
            "D3:18",
            "Melanie: [visual query: family picnic park laughing]",
        ),
        (
            "When did Caroline and Melanie go to a pride fesetival together?",
            "D12:15",
            "Caroline: We had a blast last year at the Pride fest. [visual query: friends pride festival]",
            "D10:5",
            "Caroline: Our group has regular meetings and plan events and campaigns.",
        ),
        (
            "In what ways is Caroline participating in the LGBTQ community?",
            "D10:3",
            "Caroline: I joined a new activist group called Connected LGBTQ Activists.",
            "D3:2",
            "Melanie: I'm so proud of you for spreading awareness and getting others involved in the LGBTQ community.",
        ),
        (
            "What types of pottery have Melanie and her kids made?",
            "D5:6",
            "Melanie: [visual query: pottery painted bowl intricate design; visual caption: a photo of a bowl with a black and white flower design]",
            "D8:2",
            "Melanie: We all made our own pots, it was fun and therapeutic!",
        ),
    ];

    for (query, wanted_id, wanted_text, distractor_id, distractor_text) in cases {
        let mut ranked = vec![
            (
                (distractor_id.to_string(), distractor_text.to_string()),
                1500.0,
            ),
            ((wanted_id.to_string(), wanted_text.to_string()), 0.0),
        ];
        rerank_public_benchmark_docs(query, &mut ranked);
        assert_eq!(ranked[0].0.0, wanted_id, "query: {query}");
    }
}

#[test]
fn public_benchmark_answer_support_handles_locomo_yes_no_inference() {
    assert!(public_benchmark_answer_supported_by_text(
        "Likely no, she does not refer to herself as part of it",
        "Melanie: Wow, Caroline, that sounds awesome! So glad you felt accepted and supported. Events like these are great for reminding us of how strong community can be!"
    ));
    assert!(public_benchmark_answer_supported_by_text(
        "Yes, she is supportive",
        "Melanie: Thanks, Caroline! [visual caption: a photo of a bulletin board with a rainbow flag and a don't ever be afraid to]"
    ));
}

#[test]
fn public_benchmark_answer_support_handles_locomo_counseling_paraphrase() {
    assert!(public_benchmark_answer_supported_by_text(
        "working with trans people, helping them accept themselves and supporting their mental health",
        "observation: Caroline is considering a career in counseling and mental health, particularly working with trans people to help them accept themselves and support their mental health."
    ));
}

#[test]
fn public_benchmark_evidence_target_keys_splits_locomo_joined_ids() {
    let targets = public_benchmark_evidence_target_keys(Some(&json!(["D8:6; D9:17"])));
    assert!(targets.contains("D8:6"));
    assert!(targets.contains("D9:17"));
}

#[test]
fn context_retrieval_report_counts_empty_target_answer_support_as_hit() {
    let dataset = PublicBenchmarkDatasetFixture {
        benchmark_id: "locomo".to_string(),
        benchmark_name: "synthetic".to_string(),
        version: "test".to_string(),
        split: "test".to_string(),
        description: "empty target yes/no".to_string(),
        items: vec![PublicBenchmarkDatasetFixtureItem {
            item_id: "q-empty".to_string(),
            question_id: "q-empty".to_string(),
            query: "Would Melanie be considered a member of the LGBTQ community?".to_string(),
            claim_class: "raw".to_string(),
            gold_answer: "Likely no, she does not refer to herself as part of it".to_string(),
            metadata: json!({}),
        }],
    };
    let config = PublicBenchmarkRetrievalConfig {
        longmemeval_backend: LongMemEvalRetrievalBackend::Lexical,
        sidecar_base_url: None,
        memd_base_url: None,
    };

    let report = build_context_retrieval_run_report(
        &dataset,
        5,
        "raw",
        None,
        &config,
        |_| {
            vec![(
                "D7:2".to_string(),
                "Melanie: So glad you felt accepted and supported. Events like these are great for reminding us of how strong community can be!"
                    .to_string(),
            )]
        },
        |_| BTreeSet::new(),
    )
    .expect("build report");

    assert_eq!(report.metrics.get("accuracy"), Some(&1.0));
    assert!(report.failures.is_empty());
}

#[test]
fn membench_retrieval_docs_label_recommendation_turns_without_target_ids() {
    let item = PublicBenchmarkDatasetFixtureItem {
        item_id: "mb-rec-1".to_string(),
        question_id: "mb-rec-1".to_string(),
        query: "What books have you recommended to me before?".to_string(),
        claim_class: "retrieval".to_string(),
        gold_answer: "The Darwin Awards".to_string(),
        metadata: json!({
            "message_list": [[
                {
                    "mid": 0,
                    "user": "I'm really into Seinlanguage.",
                    "assistant": "I'm glad to hear you're enjoying it."
                },
                {
                    "mid": 4,
                    "user": "I'm looking for a good book to read, aside from the ones I've mentioned earlier.",
                    "assistant": "I'm all about The Darwin Awards: Evolution in Action."
                },
                {
                    "mid": 5,
                    "user": "What's so special about this book you're suggesting?",
                    "assistant": "It's a humorous exploration of bizarre accidents."
                }
            ]]
        }),
    };
    let docs = membench_retrieval_docs(&item);
    let neutral = docs
        .iter()
        .find(|(id, _)| id == "[0,0]")
        .map(|(_, text)| text)
        .expect("neutral doc");
    let recommendation = docs
        .iter()
        .find(|(id, _)| id == "[4,0]")
        .map(|(_, text)| text)
        .expect("recommendation doc");
    let follow_up = docs
        .iter()
        .find(|(id, _)| id == "[5,0]")
        .map(|(_, text)| text)
        .expect("follow-up doc");
    assert!(
        !neutral.contains("assistant recommendation turn"),
        "neutral preference turn must not be mislabeled: {neutral}"
    );
    assert!(
        !follow_up.contains("assistant recommendation turn"),
        "recommendation follow-up must not outrank the original recommendation: {follow_up}"
    );
    assert!(
        recommendation.contains("assistant recommendation turn"),
        "recommendation turn must be searchable without target-id leakage: {recommendation}"
    );
}

#[test]
fn public_benchmark_answer_support_handles_relative_year_evidence() {
    assert!(public_benchmark_answer_supported_by_text(
        "2022",
        "(1:56 pm on 8 May, 2023) Melanie: Yeah, I painted that lake sunrise last year! It's special to me.",
    ));
    assert!(!public_benchmark_answer_supported_by_text(
        "2021",
        "(1:56 pm on 8 May, 2023) Melanie: Yeah, I painted that lake sunrise last year! It's special to me.",
    ));
}

/// j3-prep-2: mirrors the LoCoMo test above for MemBench. Pins that
/// `build_membench_full_eval_report` dispatches via `dispatch_context_retrieval_ranked("membench", ...)`
/// rather than a hardcoded lexical scorer.
#[test]
fn membench_full_eval_retrieval_honors_backend_dispatch() {
    let item = PublicBenchmarkDatasetFixtureItem {
        item_id: "mb-1".to_string(),
        question_id: "mb-1".to_string(),
        query: "the cat sat on what mat".to_string(),
        claim_class: "full-eval".to_string(),
        gold_answer: "A".to_string(),
        metadata: json!({
            "topic": "general",
            "ground_truth": "A",
            "choices": ["the mat", "the roof", "nowhere"],
            "message_list": [[
                {"mid": "m_abs", "user_message": "cat sat on the mat"},
                {"mid": "m_plain", "user_message": "cat sat on the mat"},
                {"mid": "m_cat_only", "user_message": "cat"},
                {"mid": "m_unrelated", "user_message": "the quick brown fox jumps over the lazy dog"}
            ]]
        }),
    };
    let docs = membench_retrieval_docs(&item);
    assert!(!docs.is_empty(), "membench_retrieval_docs must emit turns");
    let lex = dispatch_context_retrieval_ranked(
        "membench",
        &item.item_id,
        &item.query,
        &docs,
        "full-eval",
        &parity_cfg(PublicBenchmarkBackend::Lexical),
    );
    let rrf = dispatch_context_retrieval_ranked(
        "membench",
        &item.item_id,
        &item.query,
        &docs,
        "full-eval",
        &parity_cfg(PublicBenchmarkBackend::Rrf),
    );
    let lex_ids: Vec<&str> = lex.iter().map(|((id, _), _)| id.as_str()).collect();
    let rrf_ids: Vec<&str> = rrf.iter().map(|((id, _), _)| id.as_str()).collect();
    assert_eq!(
        lex_ids.len(),
        rrf_ids.len(),
        "membench full-eval: both backends must return every doc"
    );
    assert_ne!(
        lex_ids, rrf_ids,
        "membench full-eval: dispatcher is not routing — lexical and rrf rank identically"
    );
}

#[test]
fn dispatcher_memd_without_base_url_falls_back_to_lexical() {
    // G3 contract: Backend::Memd with no memd_base_url degrades to
    // lexical rather than panicking. Guarantees --backend memd stays
    // safe on a CLI invocation that forgot to point at a server.
    let docs = parity_fixture_docs();
    let query = "the cat sat on what mat";
    let lexical = dispatch_context_retrieval_ranked(
        "locomo",
        "item-1",
        query,
        &docs,
        "raw",
        &parity_cfg(PublicBenchmarkBackend::Lexical),
    );
    let memd_no_url = dispatch_context_retrieval_ranked(
        "locomo",
        "item-1",
        query,
        &docs,
        "raw",
        &parity_cfg(PublicBenchmarkBackend::Memd),
    );
    let lex_ids: Vec<&str> = lexical.iter().map(|((id, _), _)| id.as_str()).collect();
    let memd_ids: Vec<&str> = memd_no_url.iter().map(|((id, _), _)| id.as_str()).collect();
    assert_eq!(lex_ids, memd_ids);
}

#[test]
fn parse_membench_choices_handles_upstream_object_shape() {
    // j3-prep-3: upstream FirstAgent fixture stores `choices` as a letter-keyed
    // object `{"A": ["foo"], "B": ["foo", "bar"]}`. Regression guard — the
    // prior flat-array parser returned empty and the full-eval loop skipped
    // every item.
    let upstream = json!({
        "A": ["Dude, Where's My Country?"],
        "C": ["The Darwin Awards", "Dude, Where's My Country?"],
        "B": ["Seinlanguage"]
    });
    let rendered = parse_membench_choices(Some(&upstream));
    assert_eq!(
        rendered,
        vec![
            "A. Dude, Where's My Country?".to_string(),
            "B. Seinlanguage".to_string(),
            "C. The Darwin Awards, Dude, Where's My Country?".to_string(),
        ],
        "letters must be sorted alphabetically and arrays comma-joined"
    );
}

#[test]
fn parse_membench_choices_handles_flat_array_legacy_shape() {
    let legacy = json!(["A. Red", "B. Blue"]);
    let rendered = parse_membench_choices(Some(&legacy));
    assert_eq!(rendered, vec!["A. Red".to_string(), "B. Blue".to_string()]);
}

#[test]
fn parse_membench_choices_empty_for_null_or_missing() {
    assert!(parse_membench_choices(None).is_empty());
    assert!(parse_membench_choices(Some(&JsonValue::Null)).is_empty());
}

#[test]
fn judge_cache_key_is_deterministic_and_sensitive() {
    let a = judge_cache_key("ns", "q1", "pred1", "gpt-4o-2024-08-06", "prompt");
    let b = judge_cache_key("ns", "q1", "pred1", "gpt-4o-2024-08-06", "prompt");
    assert_eq!(a, b);
    let c = judge_cache_key("ns", "q1", "pred2", "gpt-4o-2024-08-06", "prompt");
    assert_ne!(a, c, "prediction change must change key");
    let d = judge_cache_key("ns", "q2", "pred1", "gpt-4o-2024-08-06", "prompt");
    assert_ne!(a, d, "question id change must change key");
    let e = judge_cache_key("ns", "q1", "pred1", "gpt-4o-mini", "prompt");
    assert_ne!(a, e, "grader model change must change key");
    let f = judge_cache_key("ns", "q1", "pred1", "gpt-4o-2024-08-06", "other");
    assert_ne!(a, f, "prompt change must change key");
}

#[test]
fn estimate_judge_cost_usd_matches_openai_pricing() {
    let cost = estimate_judge_cost_usd("gpt-4o-2024-08-06", 1_000_000, 0);
    assert!(
        (cost - 2.50).abs() < 1e-6,
        "1M input tokens = $2.50, got {cost}"
    );
    let cost = estimate_judge_cost_usd("gpt-4o-2024-08-06", 0, 1_000_000);
    assert!(
        (cost - 10.00).abs() < 1e-6,
        "1M output tokens = $10.00, got {cost}"
    );
    let cost = estimate_judge_cost_usd("gpt-4o-mini", 1_000_000, 0);
    assert!(
        (cost - 0.15).abs() < 1e-6,
        "mini 1M input = $0.15, got {cost}"
    );
    let cost = estimate_judge_cost_usd("gpt-4o-2024-08-06", 0, 0);
    assert_eq!(cost, 0.0);
}

#[test]
fn judge_budget_parser_rejects_zero_negative_nan() {
    assert_eq!(parse_judge_budget_str("50"), Some(50.0));
    assert_eq!(parse_judge_budget_str(" 50 "), Some(50.0));
    assert_eq!(parse_judge_budget_str("0"), None, "zero rejected");
    assert_eq!(parse_judge_budget_str("-5"), None, "negative rejected");
    assert_eq!(parse_judge_budget_str("nan"), None, "nan rejected");
    assert_eq!(parse_judge_budget_str("not-a-number"), None);
    assert_eq!(parse_judge_budget_str(""), None);
}

#[tokio::test]
async fn judge_cache_hit_serves_without_network_call() {
    let dir = std::env::temp_dir().join(format!("memd-judge-cache-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create judge cache dir");
    let key = judge_cache_key("test-ns", "q-x", "pred-x", "gpt-4o-2024-08-06", "prompt-x");
    let cache_path = dir.join(format!("{key}.json"));
    let payload = serde_json::json!({
        "content": "yes",
        "prompt_tokens": 42,
        "completion_tokens": 3,
        "grader_model": "gpt-4o-2024-08-06",
    });
    fs::write(&cache_path, serde_json::to_vec_pretty(&payload).unwrap()).expect("write cache file");
    let result = call_openai_yes_no_grader_cached_in(
        "http://127.0.0.1:1",
        "fake-key",
        "gpt-4o-2024-08-06",
        "prompt-x",
        &key,
        &dir,
    )
    .await
    .expect("cache hit should skip network");
    assert!(result.cache_hit);
    assert_eq!(result.content, "yes");
    assert_eq!(result.prompt_tokens, 42);
    assert_eq!(result.completion_tokens, 3);
    let _ = fs::remove_dir_all(&dir);
}
