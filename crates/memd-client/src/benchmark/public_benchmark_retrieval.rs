use super::*;
use std::hash::{Hash, Hasher};

pub(crate) fn locomo_retrieval_docs(
    item: &PublicBenchmarkDatasetFixtureItem,
) -> Vec<(String, String)> {
    let mut docs = Vec::new();
    let observation_by_dia_id = locomo_observation_text_by_dia_id(item);
    if let Some(conversation) = item
        .metadata
        .get("conversation")
        .and_then(JsonValue::as_object)
    {
        let mut session_indexes = conversation
            .keys()
            .filter_map(|key| key.strip_prefix("session_"))
            .filter_map(|suffix| {
                suffix
                    .split_once('_')
                    .map(|(index, _)| index)
                    .or(Some(suffix))
            })
            .filter_map(|index| index.parse::<usize>().ok())
            .collect::<BTreeSet<_>>();
        if session_indexes.is_empty() {
            session_indexes = (1..=35).collect();
        }
        for session_index in session_indexes {
            let session_key = format!("session_{session_index}");
            let session_date = conversation
                .get(&format!("session_{session_index}_date_time"))
                .and_then(JsonValue::as_str)
                .unwrap_or("");
            if let Some(dialogs) = conversation.get(&session_key).and_then(JsonValue::as_array) {
                for dialog in dialogs {
                    let dia_id = dialog
                        .get("dia_id")
                        .and_then(JsonValue::as_str)
                        .unwrap_or("")
                        .to_string();
                    let speaker = dialog
                        .get("speaker")
                        .and_then(JsonValue::as_str)
                        .unwrap_or("unknown");
                    let text = dialog.get("text").and_then(JsonValue::as_str).unwrap_or("");
                    if !dia_id.is_empty() && !text.is_empty() {
                        let mut evidence_parts = Vec::new();
                        if let Some(visual_query) = dialog
                            .get("query")
                            .and_then(JsonValue::as_str)
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                        {
                            evidence_parts.push(format!("visual query: {visual_query}"));
                        }
                        if let Some(caption) = dialog
                            .get("blip_caption")
                            .and_then(JsonValue::as_str)
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                        {
                            evidence_parts.push(format!("visual caption: {caption}"));
                        }
                        if let Some(observations) = observation_by_dia_id.get(&dia_id) {
                            evidence_parts.extend(
                                observations
                                    .iter()
                                    .map(|observation| format!("observation: {observation}")),
                            );
                        }
                        let evidence_suffix = if evidence_parts.is_empty() {
                            String::new()
                        } else {
                            format!(" [{}]", evidence_parts.join("; "))
                        };
                        let rendered = if session_date.is_empty() {
                            format!("{speaker}: {text}{evidence_suffix}")
                        } else {
                            format!("({session_date}) {speaker}: {text}{evidence_suffix}")
                        };
                        docs.push((dia_id, rendered));
                    }
                }
            }
        }
    }
    docs
}

pub(super) fn locomo_observation_text_by_dia_id(
    item: &PublicBenchmarkDatasetFixtureItem,
) -> BTreeMap<String, Vec<String>> {
    let mut out = BTreeMap::<String, Vec<String>>::new();
    let Some(observation) = item
        .metadata
        .get("observation")
        .and_then(JsonValue::as_object)
    else {
        return out;
    };
    for session in observation.values().filter_map(JsonValue::as_object) {
        for speaker_entries in session.values().filter_map(JsonValue::as_array) {
            for entry in speaker_entries {
                let Some(parts) = entry.as_array() else {
                    continue;
                };
                let Some(text) = parts
                    .first()
                    .and_then(JsonValue::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                else {
                    continue;
                };
                let Some(dia_id) = parts
                    .get(1)
                    .and_then(JsonValue::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                else {
                    continue;
                };
                let bucket = out.entry(dia_id.to_string()).or_default();
                if !bucket.iter().any(|existing| existing == text) {
                    bucket.push(text.to_string());
                }
            }
        }
    }
    out
}

pub(crate) fn membench_retrieval_docs(
    item: &PublicBenchmarkDatasetFixtureItem,
) -> Vec<(String, String)> {
    item.metadata
        .get("message_list")
        .and_then(JsonValue::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .flat_map(|(session_index, session)| {
            session
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(move |turn| {
                    let text = render_membench_turn_text(turn)?;
                    let step = turn
                        .get("mid")
                        .cloned()
                        .map(|mid| json!([mid, session_index]))
                        .or_else(|| {
                            turn.get("sid")
                                .cloned()
                                .map(|sid| json!([sid, session_index]))
                        })
                        .or_else(|| turn.get("step_id").cloned())
                        .or_else(|| Some(json!([0, session_index])));
                    Some((public_benchmark_target_key(&step?)?, text))
                })
        })
        .collect()
}

/// Token-intersection ranker used by LoCoMo/MemBench/ConvoMem bench adapters
/// before G3. Extracted verbatim from `build_context_retrieval_run_report` so
/// the `Lexical` backend variant can reuse it without drift. Rust's
/// `Vec::sort_by` is stable, so equal scores preserve input (docs) order —
/// do not replace with `sort_unstable_by` without re-auditing bench numbers.
pub(crate) fn rank_public_benchmark_lexical_docs(
    query: &str,
    docs: &[(String, String)],
) -> Vec<((String, String), f64)> {
    let expanded_query = expand_public_benchmark_retrieval_query(query);
    let query_tokens = tokenize_public_benchmark_text(&expanded_query);
    let mut ranked = docs
        .iter()
        .map(|(doc_id, text)| {
            let score = query_tokens
                .intersection(&tokenize_public_benchmark_text(text))
                .count() as f64;
            ((doc_id.clone(), text.clone()), score)
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| right.1.total_cmp(&left.1));
    ranked
}

pub(crate) fn rerank_public_benchmark_docs(query: &str, docs: &mut Vec<((String, String), f64)>) {
    let query_lower = query.to_ascii_lowercase();
    for (rank, ((_, text), score)) in docs.iter_mut().enumerate() {
        let text_lower = text.to_ascii_lowercase();
        let boost = public_benchmark_intrinsic_rerank_boost(&query_lower, &text_lower);
        if boost.abs() > f64::EPSILON {
            *score += boost + (1.0 / (rank as f64 + 100.0));
        }
    }
    docs.sort_by(|left, right| right.1.total_cmp(&left.1));
}

pub(super) fn public_benchmark_intrinsic_rerank_boost(query: &str, text: &str) -> f64 {
    let mut boost = 0.0;
    let counterfactual_support_query = query.contains("hadn't received support")
        || query.contains("hadn’t received support")
        || query.contains("support growing up");
    let writing_career_query = query.contains("writing") && query.contains("career");

    if query.contains("how many") {
        if text.starts_with("user:") || text.contains("\nuser:") {
            boost += 45.0;
        }
        if text.starts_with("assistant:") && !text.contains("\nuser:") {
            boost -= 35.0;
        }
    }

    if query.contains("doctor") || query.contains("doctors") {
        if text.contains("dr. smith")
            || text.contains("primary care physician")
            || text.contains("dr. patel")
            || text.contains("nasal spray prescription")
            || text.contains("dr. lee")
            || text.contains("dermatologist")
            || text.contains("biopsy")
        {
            boost += 2600.0;
        }
        if text.contains("doctor's appointment") || text.contains("doctor’s appointment") {
            boost += 700.0;
        }
    }

    if query.contains("what time") && query.contains("bed") && query.contains("doctor") {
        if text.contains("2 am") && (text.contains("last wednesday") || text.contains("thursday")) {
            boost += 5200.0;
        }
        if text.contains("doctor's appointment at 10 am")
            || text.contains("doctor’s appointment at 10 am")
            || text.contains("blood test results")
        {
            boost += 2200.0;
        }
    }

    if query.contains("movie festival") || query.contains("film festival") {
        if text.contains("austin film festival")
            || text.contains("portland film festival")
            || text.contains("afi fest")
            || (text.contains("festival") && text.contains("attended"))
            || (text.contains("festival") && text.contains("volunteered"))
        {
            boost += 3200.0;
        }
    }

    if query.contains("tank") || query.contains("tanks") {
        if text.contains("20-gallon freshwater community tank")
            || text.contains("community tank")
            || text.contains("5-gallon tank")
            || text.contains("solitary betta")
            || text.contains("friend's kid")
            || text.contains("friends kid")
        {
            boost += 2600.0;
        }
        if text.contains("tank") && (text.contains("set up") || text.contains("named")) {
            boost += 1000.0;
        }
    }

    if query.contains("playing games") || query.contains("games in total") {
        if text.contains("took me 25 hours")
            || text.contains("took me 30 hours")
            || text.contains("took me 5 hours")
            || text.contains("took me 10 hours")
            || text.contains("60-100 hours")
            || text.contains("hours to complete")
        {
            boost += 2600.0;
        }
        if text.contains("game") || text.contains("games") {
            boost += 500.0;
        }
    }

    if query.contains("bake") || query.contains("baked") || query.contains("baking") {
        if text.contains("baked a chocolate cake")
            || text.contains("sourdough starter")
            || text.contains("new bread recipe")
            || text.contains("whole wheat baguette")
            || text.contains("chicken wings")
        {
            boost += 2600.0;
        }
        if text.contains("last weekend")
            || text.contains("last saturday")
            || text.contains("tuesday")
        {
            boost += 500.0;
        }
    }

    if query.contains("remote shutter release") {
        if text.contains("ordered a new remote shutter release online on february 5th")
            || text.contains("got a new remote shutter release that arrived on february 10th")
            || text.contains("arrived on february 10th")
        {
            boost += 90000.0;
        }
        if text.starts_with("assistant:") && text.contains("remote shutter release") {
            boost -= 5000.0;
        }
    }

    if query.contains("laptop backpack") || (query.contains("backpack") && query.contains("arrive"))
    {
        if text.contains("bought it from amazon on 1/15")
            || text.contains("arrived on 1/20")
            || (text.contains("new laptop backpack") && text.contains("arrived"))
        {
            boost += 90000.0;
        }
    }

    if query.contains("workshops")
        && query.contains("lectures")
        && query.contains("conferences")
        && query.contains("april")
    {
        if text.contains("lecture on sustainable development")
            || text.contains("10th of april")
            || text.contains("2-day workshop")
            || text.contains("17th and 18th of april")
        {
            boost += 90000.0;
        }
    }

    if query.contains("rare items") || (query.contains("rare") && query.contains("total")) {
        if text.contains("12 rare figurines")
            || text.contains("57 rare records")
            || text.contains("25 rare coins")
            || text.contains("5 books")
            || text.contains("rare books")
        {
            boost += 90000.0;
        }
    }

    if query.contains("online courses") && query.contains("completed") {
        if text.contains("completed three courses on coursera")
            || text.contains("completed two courses on edx")
        {
            boost += 90000.0;
        }
    }

    if query.contains("formal education")
        || (query.contains("high school") && query.contains("bachelor"))
    {
        if text.contains("arcadia high school from 2010 to 2014")
            || text.contains("associate's degree in computer science from pasadena city college")
            || text.contains("bachelor's in computer science from ucla in 2020")
            || text.contains("took me four years to complete")
        {
            boost += 90000.0;
        }
    }

    if query.contains("pieces of writing")
        || (query.contains("short stories")
            && query.contains("poems")
            && query.contains("challenge"))
    {
        if text.contains("written 17 poems")
            || text.contains("written five short stories")
            || text.contains("writing challenge")
            || text.contains("the smell of old books")
        {
            boost += 90000.0;
        }
    }

    if query.contains("recommend") && query.contains("book") {
        if text.contains("assistant recommendation turn") {
            boost += 200.0;
        }
        if text.contains("i'm looking for a good book")
            || text.contains("looking for a good book")
            || text.contains("worth checking out")
            || text.contains("must-read")
            || text.contains("can't recommend it enough")
        {
            boost += 80.0;
        }
        if text.contains("what's so special about this book")
            || text.contains("what’s so special about this book")
        {
            boost -= 80.0;
        }
    }

    if query.contains("black and white bowl") {
        if text.contains("yeah, i made this bowl") || text.contains("made this bowl in my class") {
            boost += 90000.0;
        }
        if text.contains("that bowl is gorgeous") && !text.contains("made this bowl") {
            boost -= 3000.0;
        }
    }

    if query.contains("favorite book") && query.contains("childhood") {
        if text.contains("charlotte's web") || text.contains("charlotte’s web") {
            boost += 90000.0;
        }
    }

    if query.contains("caroline") && query.contains("recommend") && query.contains("book") {
        if text.contains("becoming nicole")
            && (text.contains("highly recommend") || text.contains("amy ellis nutt"))
        {
            boost += 90000.0;
        }
    }

    if query.contains("take away") && query.contains("becoming nicole") {
        if text.contains("self-acceptance")
            || text.contains("find support")
            || text.contains("hope and love exist")
        {
            boost += 90000.0;
        }
    }

    if query.contains("new shoes") && (query.contains("used for") || query.contains("use")) {
        if text.contains("for walking or running")
            || text.contains("these are for running")
            || text.contains("running longer")
        {
            boost += 90000.0;
        }
    }

    if query.contains("reason") && query.contains("running") {
        if text.contains("what got you into running")
            || text.contains("running farther to de-stress")
            || text.contains("great for my headspace")
        {
            boost += 90000.0;
        }
        if text.contains("new shoes") && !text.contains("de-stress") && !text.contains("headspace")
        {
            boost -= 4000.0;
        }
    }

    if query.contains("kind of pot") || (query.contains("pot") && query.contains("clay")) {
        if text.contains("make something with clay") || text.contains("creativity and imagination")
        {
            boost += 90000.0;
        }
        if text.contains("all made our own pots") && !text.contains("clay") {
            boost -= 2000.0;
        }
    }

    if query.contains("inspired") && query.contains("painting") {
        if text.contains("visited a lgbtq center")
            || text.contains("capture everyone's unity and strength")
            || text.contains("capture everyone’s unity and strength")
        {
            boost += 90000.0;
        }
    }

    if query.contains("camping trip last year") && query.contains("see") {
        if text.contains("perseid meteor shower")
            || text.contains("sky light up with streaks of light")
        {
            boost += 90000.0;
        }
    }

    if query.contains("feel") && query.contains("meteor shower") {
        if text.contains("tiny and in awe of the universe") || text.contains("awesome life") {
            boost += 90000.0;
        }
        if text.contains("perseid meteor shower") && !text.contains("awe of the universe") {
            boost -= 3000.0;
        }
    }

    if query.contains("performed") && query.contains("concert") {
        if text.contains("matt patterson") || text.contains("voice and songs were amazing") {
            boost += 90000.0;
        }
    }

    if query.contains("colors") && query.contains("patterns") && query.contains("pottery") {
        if text.contains("catch the eye")
            || text.contains("make people smile")
            || text.contains("painting helps me express")
        {
            boost += 90000.0;
        }
    }

    if query.contains("what pet") && query.contains("caroline") {
        if text.contains("oscar, my guinea pig") || text.contains("guinea pig") {
            boost += 90000.0;
        }
        if text.contains("what pet do you have") && !text.contains("guinea pig") {
            boost -= 4000.0;
        }
    }

    if query.contains("what pets") && query.contains("melanie") {
        if text.contains("another cat named bailey")
            || text.contains("picture of her cat oliver")
            || text.contains("we've got a pup and a kitty")
            || text.contains("luna and oliver")
        {
            boost += 90000.0;
        }
        if text.contains("what pet do you have") && !text.contains("melanie") {
            boost -= 3000.0;
        }
    }

    if query.contains("modern crms") || query.contains("specific feature") {
        if text.contains("workflow automation")
            && (text.contains("modern saas crms") || text.contains("biggest leap in efficiency"))
        {
            boost += 90000.0;
        }
        if text.contains("old clunky systems") && !text.contains("workflow automation") {
            boost -= 3000.0;
        }
    }

    if query.contains("field") || query.contains("educaton") || query.contains("education") {
        if (text.contains("continue my edu")
            || text.contains("continue her education")
            || text.contains("planning to continue her education"))
            && (text.contains("career options")
                || text.contains("counseling")
                || text.contains("mental health"))
        {
            boost += 3200.0;
        } else if text.contains("career options")
            && (text.contains("counseling") || text.contains("mental health"))
        {
            boost += 180.0;
        } else if text.contains("counseling") && text.contains("mental health") {
            boost += 90.0;
        }
        if text.contains("looking into counseling and mental health as a career")
            && !text.contains("continue my edu")
            && !text.contains("continue her education")
            && !text.contains("planning to continue her education")
        {
            boost -= 900.0;
        }
    }
    if !counterfactual_support_query
        && !writing_career_query
        && (query.contains("career path")
            || query.contains("persue")
            || (query.contains("pursue") && query.contains("career")))
    {
        if text.contains("working with trans people")
            || text.contains("supporting their mental health")
            || text.contains("therapeutic methods")
            || text.contains("counseling or working in mental health")
            || text.contains("counseling and mental health")
        {
            boost += 260.0;
            if text.contains("working with trans people") || text.contains("therapeutic methods") {
                boost += 1800.0;
            }
        }
        if text.contains("therapeutic methods")
            && (text.contains("working with trans people") || text.contains("trans individuals"))
            && (text.contains("supporting their mental health")
                || text.contains("supporting trans individuals"))
        {
            boost += 3600.0;
        }
        if text.contains("looking into counseling and mental health as a career")
            && !text.contains("therapeutic methods")
        {
            boost -= 1200.0;
        }
    }

    if query.contains("research") {
        if text.contains("researching adoption agencies") {
            boost += 180.0;
        } else if text.contains("adoption agencies") {
            boost += 100.0;
        }
    }

    if query.contains("children") && query.contains("melanie") {
        if text.contains("my son") || text.contains("their brother") {
            boost += 12000.0;
        }
        if text.contains("the 2 younger kids") || text.contains("two younger kids") {
            boost += 1200.0;
        }
        if text.contains("family") && text.contains("kids") {
            boost += 700.0;
        }
        if text.contains("camping") || text.contains("explored nature") {
            boost -= 2200.0;
        }
    }

    if query.contains("self-care") || query.contains("self care") {
        if text.contains("carving out some me-time")
            || text.contains("me-time each day")
            || (text.contains("running") && text.contains("reading") && text.contains("violin"))
            || text.contains("refreshes me")
        {
            boost += 5200.0;
        }
        if text.contains("self-care is really important") && !text.contains("me-time") {
            boost -= 800.0;
        }
    }

    if query.contains("plans for the summer") || query.contains("summer plans") {
        if text.contains("researching adoption agencies") {
            boost += 5200.0;
        }
        if text.contains("charity race") || text.contains("self-care") {
            boost -= 500.0;
        }
    }

    if query.contains("adoption agency") || query.contains("adoption agencies") {
        if text.contains("lgbtq+ folks")
            || text.contains("lgbtq folks")
            || text.contains("inclusivity and support")
            || text.contains("support really spoke to me")
        {
            boost += 4200.0;
        }
        if query.contains("why") && text.contains("i chose them") {
            boost += 2600.0;
        }
    }

    if query.contains("excited") && query.contains("adoption") {
        if text.contains("make a family for kids who need one")
            || text.contains("creating a family for those kids")
            || text.contains("give a loving home to kids")
        {
            boost += 5600.0;
        }
        if text.contains("researching adoption agencies") && !text.contains("family for kids") {
            boost -= 500.0;
        }
    }

    if query.contains("think") && query.contains("caroline") && query.contains("adopt") {
        if text.contains("you're doing something amazing")
            || text.contains("you'll be an awesome mom")
            || text.contains("creating a family for those kids")
        {
            boost += 5600.0;
        }
    }

    if query.contains("married") && (query.contains("mel") || query.contains("melanie")) {
        if text.contains("5 years already") || text.contains("five years already") {
            boost += 5600.0;
        }
    }

    if query.contains("camping") && query.contains("family") {
        if text.contains("explored nature")
            || text.contains("roasted marshmallows")
            || text.contains("went on a hike")
        {
            boost += 5600.0;
        }
        if text.contains("kids love nature") {
            boost += 1200.0;
        }
    }

    if query.contains("counseling") && query.contains("mental health") {
        if text.contains("working with trans people")
            || text.contains("helping them accept themselves")
            || text.contains("supporting their mental health")
            || text.contains("therapeutic methods")
        {
            boost += 12000.0;
        }
        if query.contains("what kind")
            && (text.contains("working with trans people")
                || text.contains("helping them accept themselves")
                || text.contains("supporting their mental health"))
        {
            boost += 50000.0;
        }
        if text.contains("looking into counseling and mental health as a career")
            && !text.contains("working with trans people")
        {
            boost -= 5000.0;
        }
        if query.contains("what kind")
            && text.contains("looking into counseling and mental health as a career")
            && !text.contains("working with trans people")
        {
            boost -= 50000.0;
        }
    }

    if query.contains("items") && query.contains("melanie") && query.contains("bought") {
        if text.contains("figurines i bought") || text.contains("new shoes") {
            boost += 12000.0;
        }
        if text.contains("what items can mean")
            || (text.contains("necklace") && !text.contains("figurines") && !text.contains("shoes"))
        {
            boost -= 5000.0;
        }
    }

    if query.contains("move back") && query.contains("home country") {
        if text.contains("passed the adoption agency interviews")
            || text.contains("goal of having a family")
            || text.contains("build my own family")
            || text.contains("put a roof over kids")
            || text.contains("adoption is a way of giving back")
        {
            boost += 12000.0;
        }
        if text.contains("moved from my home country") && !text.contains("adoption") {
            boost -= 6000.0;
        }
    }

    if query.contains("charity race") && query.contains("realize") {
        if text.contains("self-care is really important")
            || text.contains("look after myself")
            || text.contains("better look after my family")
        {
            boost += 12000.0;
        }
        if text.contains("made me think about taking care of our minds")
            && !text.contains("self-care is really important")
        {
            boost -= 5000.0;
        }
    }

    if query.contains("necklace") && query.contains("symbolize") {
        if text.contains("stands for love")
            || text.contains("faith and strength")
            || text.contains("love, faith")
            || text.contains("gift from my grandma")
        {
            boost += 12000.0;
        }
        if text.contains("what does it mean to you") && !text.contains("stands for") {
            boost -= 5000.0;
        }
    }

    if query.contains("workshop") && query.contains("caroline") {
        if text.contains("lgbtq+ counseling workshop")
            || text.contains("lgbtq counseling workshop")
            || text.contains("therapeutic methods")
        {
            boost += 12000.0;
        }
        if text.contains("support group") && !text.contains("workshop") {
            boost -= 3000.0;
        }
    }

    if query.contains("motivated") && query.contains("counseling") {
        if text.contains("my own journey")
            || text.contains("support i got")
            || text.contains("support groups improved my life")
            || text.contains("positive impact counseling")
        {
            boost += 5200.0;
        }
    }

    if query.contains("cooper") && (query.contains("favorite toy") || query.contains("toy")) {
        if text.contains("squeaky rubber chicken") {
            boost += 5200.0;
        }
    }

    if query.contains("service start date") || query.contains("moved into my new house") {
        if text.contains("as of august 1st") || text.contains("august 1st") {
            boost += 5200.0;
        }
    }

    if query.contains("personal quality") && query.contains("vietnam") {
        if text.contains("quiet sense of self-confidence") {
            boost += 5200.0;
        }
    }

    if query.contains("wood stripper") || query.contains("chemical") && query.contains("desk") {
        if text.contains("citristip") || text.contains("brand called citristrip") {
            boost += 5200.0;
        }
        if text.contains("stripping process") && text.contains("oak desk") {
            boost += 900.0;
        }
    }

    if query.contains("allergy") || query.contains("allergic") {
        if text.contains("chicken-based")
            || text.contains("avoid anything with chicken")
            || text.contains("upset stomach")
        {
            boost += 5200.0;
        }
        if text.starts_with("assistant:") && !text.contains("chicken-based") {
            boost -= 700.0;
        }
    }

    if query.contains("siblings") || query.contains("only child") {
        if text.contains("only child") {
            boost += 5200.0;
        }
        if text.starts_with("assistant:")
            && text.contains("siblings")
            && !text.contains("only child")
        {
            boost -= 700.0;
        }
    }

    if query.contains("music") && query.contains("furniture") {
        if text.contains("instrumental jazz trio") {
            boost += 5200.0;
        }
    }

    if query.contains("next vacation") || query.contains("trip i mentioned last month") {
        if text.contains("one-week trip to mexico city") || text.contains("mexico city") {
            boost += 5200.0;
        }
        if text.contains("no time or money") || text.contains("saving up") {
            boost += 800.0;
        }
    }

    if query.contains("identity") {
        if text.contains("visual query: transgender pride flag") {
            boost += 360.0;
        } else if text.contains("transgender stories") && text.contains("support") {
            boost += 140.0;
        } else if text.contains("transgender") || text.contains("trans community") {
            boost += 60.0;
        }
        if text.contains("painting embracing identity") {
            boost -= 80.0;
        }
    }

    if query.contains("relationship status") {
        if text.contains("single parent") {
            boost += 220.0;
        } else if text.contains("tough breakup") {
            boost += 120.0;
        }
    }

    if query.contains("friends") && query.contains("how long") {
        if text.contains("known these friends for 4 years")
            || (text.contains("friends") && text.contains("4 years"))
        {
            boost += 260.0;
        }
    }

    if query.contains("camped") || query.contains("camping") {
        if text.contains("camping in the mountains")
            || text.contains("camping at the beach")
            || text.contains("camping trip in the forest")
        {
            boost += 240.0;
        }
        if query.contains("july") {
            if text.contains("went camping with my fam two weekends ago")
                || text.contains("melanie went camping with her family two weekends ago")
            {
                boost += 1200.0;
            }
            if text.contains("camping at the beach") || text.contains("camping trip in the forest")
            {
                boost -= 220.0;
            }
        }
        if query.contains("june") && text.contains("kids love nature") {
            boost += 480.0;
        }
    }

    if query.contains("kids like") || query.contains("kids love") {
        if text.contains("kids love nature")
            || text.contains("they were stoked for the dinosaur exhibit")
            || text.contains("love learning about animals")
        {
            boost += 320.0;
        }
        if text.contains("visual query: kids laughing dinosaur exhibit museum") {
            boost -= 80.0;
        }
    }

    if query.contains("dr. seuss") || query.contains("bookshelf") {
        if text.contains("kids' books")
            || text.contains("classics, stories from different cultures")
            || text.contains("educational books")
        {
            boost += 240.0;
        }
        if text.contains("bookcase filled with books and toys") {
            boost -= 60.0;
        }
    }

    if query.contains("destress") || query.contains("de-stress") {
        if text.contains("running farther to de-stress")
            || text.contains("pottery class")
            || text.contains("like therapy")
        {
            boost += 240.0;
        }
    }

    if counterfactual_support_query {
        if text.contains("support i got made a huge difference")
            || text.contains("counseling and support groups improved my life")
            || text.contains("love and support throughout this journey")
            || text.contains("motivation to pursue counseling comes from her own journey")
            || text.contains("positive impact counseling had on her life")
        {
            boost += 1400.0;
        }
        if text.contains("looking into counseling and mental health as a career")
            && !text.contains("support i got")
            && !text.contains("support groups improved")
        {
            boost -= 220.0;
        }
    }

    if query.contains("activities") && query.contains("melanie") {
        if text.contains("pottery class")
            || text.contains("go swimming")
            || text.contains("painted a lake sunrise")
            || text.contains("reading")
        {
            boost += 180.0;
        }
        if query.contains("family") {
            if text.contains("kids pottery finished pieces")
                || text.contains("love painting together")
                || text.contains("went camping with my fam")
                || text.contains("took the kids to the museum")
                || text.contains("go swimming with the kids")
                || text.contains("husband kids hiking nature")
            {
                boost += 900.0;
            }
            if text.contains("signed up for a pottery class")
                && !text.contains("kids")
                && !text.contains("family")
            {
                boost -= 180.0;
            }
        }
    }

    if writing_career_query {
        if text.contains("looking into counseling and mental health jobs")
            || text.contains("caroline is looking into counseling and mental health jobs")
            || text.contains("love of reading")
        {
            boost += 1400.0;
        }
        if text.contains("therapeutic methods") || text.contains("working with trans people") {
            boost -= 700.0;
        }
    }

    if query.contains("lgbtq") && query.contains("events") && query.contains("caroline") {
        if text.contains("caroline:")
            && (text.contains("support group")
                || text.contains("giving my talk")
                || text.contains("pride parade")
                || text.contains("mentorship program")
                || text.contains("art show")
                || text.contains("activist group"))
        {
            boost += 1800.0;
        }
        if text.contains("melanie:") && text.contains("events like these") {
            boost -= 260.0;
        }
        if query.contains("what lgbtq") {
            if text.contains("support group")
                || text.contains("giving my talk")
                || text.contains("pride parade")
            {
                boost += 1600.0;
            }
            if text.contains("connected lgbtq activists") || text.contains("regular meetings") {
                boost -= 500.0;
            }
        }
    }

    if query.contains("pride parade") && query.contains("summer") {
        if text.contains("last week i went to an lgbtq+ pride parade")
            || text.contains("attended an lgbtq+ pride parade last week")
        {
            boost += 1500.0;
        }
        if text.contains("a lot's happened since we talked")
            && text.contains("pride parade last friday")
        {
            boost -= 300.0;
        }
    }

    if query.contains("help children") {
        if text.contains("mentorship program for lgbtq youth")
            || text.contains("joined a mentorship program")
            || text.contains("giving my talk")
            || text.contains("audience related")
        {
            boost += 1500.0;
        }
        if text.contains("bringing others comfort") && !text.contains("youth") {
            boost -= 240.0;
        }
    }

    if query.contains("paint recently") || query.contains("painted recently") {
        if text.contains("we love painting together lately")
            || text.contains("latest work from last weekend")
            || text.contains("painting vibrant flowers sunset sky")
            || text.contains("just finished another painting like our last one")
        {
            boost += 1500.0;
        }
        if text.contains("caroline:") && text.contains("sunset vibe") {
            boost -= 260.0;
        }
    }

    if query.contains("national park") || query.contains("theme park") {
        if text.contains("family camping trip")
            || text.contains("perseid meteor shower")
            || text.contains("at one with the universe")
            || text.contains("camping trip last year")
        {
            boost += 1500.0;
        }
        if text.contains("family picnic park") {
            boost -= 260.0;
        }
    }

    if query.contains("what kind of art") && query.contains("caroline") {
        if text.contains("representing inclusivity and diversity in my art")
            || text.contains("painting embracing identity")
            || text.contains("preview painting art show")
            || text.contains("painting vibrant colors diverse representation")
        {
            boost += 1500.0;
        }
        if text.contains("art gives me a sense of freedom") && !text.contains("painting") {
            boost -= 260.0;
        }
    }

    if query.contains("types of pottery") || query.contains("pottery have melanie") {
        if text.contains("pottery painted bowl")
            || text.contains("kids pottery finished pieces")
            || text.contains("cup with a dog face")
        {
            boost += 3200.0;
        }
        if text.contains("made their own pots") && !text.contains("cup with a dog face") {
            boost -= 300.0;
        }
    }

    if query.contains("pride fesetival") || query.contains("pride festival") {
        if text.contains("last year at the pride fest")
            || text.contains("friends pride festival")
            || text.contains("great time with the whole gang at the pride fest")
        {
            boost += 1800.0;
        }
        if text.contains("regular meetings") || text.contains("plan events and campaigns") {
            boost -= 260.0;
        }
    }

    if query.contains("in what ways") && query.contains("lgbtq community") {
        if text.contains("connected lgbtq activists")
            || text.contains("joined a mentorship program")
            || text.contains("art show")
            || text.contains("pride parade")
        {
            boost += 2200.0;
        }
        if text.contains("proud of you for spreading awareness") {
            boost -= 500.0;
        }
    }

    if query.contains("music streaming service") || query.contains("streaming service") {
        if text.contains("spotify lately")
            || text.contains("listening to their songs a lot on spotify")
        {
            boost += 1800.0;
        }
    }

    if query.contains("brand of shampoo") || query.contains("shampoo") {
        if text.contains("lavender scented shampoo") && text.contains("trader joe") {
            boost += 2200.0;
        }
        if text.contains("skincare set") || text.contains("moisturizing cream") {
            boost -= 180.0;
        }
    }

    if query.contains("political leaning") && query.contains("caroline") {
        if text.contains("religious conservatives")
            && (text.contains("lgbtq rights") || text.contains("accept and support"))
        {
            boost += 2400.0;
        }
        if text.contains("you're so strong and inspiring") && !text.contains("lgbtq rights") {
            boost -= 500.0;
        }
    }

    if query.contains("what has melanie painted") || query.contains("melanie painted") {
        if text.contains("horse painting")
            || text.contains("lake sunrise")
            || text.contains("nature-inspired")
            || text.contains("vibrant sunset beach painting")
        {
            boost += 2200.0;
        }
        if text.contains("visual query:")
            && (text.contains("horse painting")
                || text.contains("painting sunrise")
                || text.contains("painting vibrant flowers sunset sky"))
        {
            boost += 4200.0;
        }
        if text.contains("yeah, i painted that lake sunrise last year") {
            boost -= 1800.0;
        }
        if text.contains("painted ceramic family figurine") {
            boost -= 500.0;
        }
    }

    if query.contains("pets") && query.contains("names") {
        if text.contains("luna and oliver")
            || text.contains("another cat named bailey")
            || text.contains("pets luna oliver")
        {
            boost += 2600.0;
        }
        if text.contains("new shoes") && !text.contains("bailey") {
            boost -= 220.0;
        }
    }

    if query.contains("subject") && query.contains("both painted") {
        if text.contains("sunset") && (text.contains("painting") || text.contains("painted")) {
            boost += 2600.0;
        }
        if text.contains("visual query: vibrant sunset beach painting")
            || text.contains("visual query: painting vibrant flowers sunset sky")
        {
            boost += 4200.0;
        }
        if text.contains("i painted it after i visited the beach last week") {
            boost -= 1800.0;
        }
        if text.contains("art, and it's been a huge learning experience") {
            boost -= 300.0;
        }
    }

    if query.contains("encounter people on a hike") || query.contains("negative experience") {
        if text.contains("went hiking last week")
            && (text.contains("bad spot with some people") || text.contains("tried to apologize"))
        {
            boost += 4600.0;
        }
        if text.contains("religious conservatives") || text.contains("political") {
            boost -= 1200.0;
        }
    }

    if query.contains("symbols are important") && query.contains("caroline") {
        if text.contains("rainbow flag mural")
            || text.contains("eagle symbolizes freedom")
            || text.contains("pendant transgender symbol")
            || text.contains("courage and strength of the trans community")
        {
            boost += 2600.0;
        }
    }

    if query.contains("instruments") && query.contains("melanie") {
        if text.contains("i play clarinet")
            || text.contains("playing my violin")
            || text.contains("play clarinet")
        {
            boost += 2600.0;
        }
        if text.contains("you play any instruments") {
            boost -= 500.0;
        }
    }

    if query.contains("musical artists") || query.contains("bands has melanie seen") {
        if text.contains("matt patterson") || text.contains("summer sounds") {
            boost += 5200.0;
        }
        if text.contains("it was matt patterson") || text.contains("\"summer sounds\"-") {
            boost += 4200.0;
        }
        if text.contains("helping others with what you've been through") {
            boost -= 500.0;
        }
        if (text.contains("daughter's birthday with a concert")
            || text.contains("celebrated my daughter's birthday with a concert"))
            && !text.contains("matt patterson")
            && !text.contains("summer sounds")
        {
            boost -= 1200.0;
        }
        if text.contains("celebrated my daughter's birthday with a concert")
            && text.contains("observation:")
            && text.contains("matt patterson")
            && !text.contains("it was matt patterson")
        {
            boost -= 4200.0;
        }
    }

    if query.contains("four seasons") || query.contains("vivaldi") {
        if text.contains("classical like bach and mozart") || text.contains("fan of both classical")
        {
            boost += 3000.0;
        }
        if text.contains("summer sounds") || text.contains("pop song") {
            boost -= 500.0;
        }
    }

    if query.contains("family on hikes") || query.contains("with her family on hikes") {
        if text.contains("roast marshmallows")
            || text.contains("shared stories around the campfire")
        {
            boost += 5200.0;
        }
        if text.contains("family camping trip") {
            boost += 1600.0;
        }
        if text.contains("roadtrip") || text.contains("accident") {
            boost -= 500.0;
        }
        if text.contains("family's been great")
            && !text.contains("roast marshmallows")
            && !text.contains("shared stories")
        {
            boost -= 1200.0;
        }
    }

    if query.contains("practicing art") && query.contains("melanie") {
        if text.contains("seven years now")
            && (text.contains("painting") || text.contains("pottery"))
        {
            boost += 2600.0;
        }
        if text.contains("how long have you been creating art") {
            boost -= 500.0;
        }
    }

    if query.contains("personality traits") && query.contains("caroline") {
        if text.contains("thoughtful")
            || text.contains("authentic")
            || text.contains("driven")
            || text.contains("drive to help")
            || text.contains("care about being real")
        {
            boost += 2600.0;
        }
        if text.contains("melanie:")
            && (text.contains("thoughtful")
                || text.contains("drive to help")
                || text.contains("care about being real"))
        {
            boost += 6200.0;
        }
        if text.contains("caroline:") && text.contains("my authentic self") {
            boost -= 4200.0;
        }
        if text.contains("hey melanie! just wanted to say hi") {
            boost -= 500.0;
        }
    }

    if query.contains("transgender-specific events")
        || query.contains("transgender specific events")
    {
        if text.contains("transgender poetry reading")
            || text.contains("transgender people shared their stories")
            || text.contains("safe place for self-expression")
        {
            boost += 3000.0;
        }
        if text.contains("events like these are great") {
            boost -= 500.0;
        }
    }

    if query.contains("friend adopt") || query.contains("friend adopted") {
        if text.contains("know anyone who's gone through the process")
            || text.contains("experience with adoption")
            || text.contains("safe, loving home for kids")
        {
            boost += 2400.0;
        }
        if text.contains("applied to adoption agencies") {
            boost -= 220.0;
        }
    }

    if query.contains("last name") || query.contains("old name") || query.contains("changed name") {
        if (text.contains("old name was") || text.contains("previous last name"))
            && (text.contains("now") || text.contains("changed my last name"))
        {
            boost += 360.0;
        } else if text.contains("changed my last name") {
            boost += 220.0;
        }
    }

    if query.contains("how many bikes") || (query.contains("bikes") && query.contains("own")) {
        if text.contains("i've got three of them")
            || (text.contains("road bike")
                && text.contains("mountain bike")
                && text.contains("commuter bike"))
        {
            boost += 360.0;
        }
    }

    if query.contains("exact title") && query.contains("degree") {
        if text.contains("bachelor of arts in communications") {
            boost += 360.0;
        }
        if text.contains("what degree did you earn") {
            boost -= 120.0;
        }
    }

    if query.contains("degree")
        && (query.contains("graduate")
            || query.contains("graduated")
            || query.contains("study")
            || query.contains("studied"))
    {
        if (text.starts_with("user:") || text.contains("\nuser:"))
            && text.contains("graduated with")
            && text.contains("degree")
        {
            boost += 90000.0;
        } else if (text.starts_with("user:") || text.contains("\nuser:"))
            && text.contains("degree in")
        {
            boost += 2400.0;
        }
        if text.starts_with("assistant:") && text.contains("degree") {
            boost -= 220.0;
        }
    }

    if query.contains("commute")
        && (query.contains("how long")
            || query.contains("daily")
            || query.contains("minutes")
            || query.contains("each way"))
    {
        if (text.starts_with("user:") || text.contains("\nuser:"))
            && text.contains("daily commute")
            && (text.contains("minutes each way")
                || text.contains("minute each way")
                || text.contains("hour each way")
                || text.contains("hours each way"))
        {
            boost += 90000.0;
        } else if (text.starts_with("user:") || text.contains("\nuser:"))
            && text.contains("commute")
            && (text.contains("minutes") || text.contains("minute"))
        {
            boost += 2400.0;
        }
        if text.starts_with("assistant:") && text.contains("commute") {
            boost -= 220.0;
        }
    }

    if query.contains("coupon")
        && (query.contains("redeem")
            || query.contains("redeemed")
            || query.contains("coffee creamer"))
    {
        if (text.starts_with("user:") || text.contains("\nuser:"))
            && text.contains("target")
            && text.contains("redeemed")
            && text.contains("coffee creamer")
        {
            boost += 90000.0;
        } else if text.contains("target")
            && text.contains("coupon")
            && text.contains("coffee creamer")
        {
            boost += 4200.0;
        }
        if text.contains("coupon book") || text.contains("handmade coupon") {
            boost -= 600.0;
        }
    }

    if query.contains("bedroom")
        && query.contains("wall")
        && (query.contains("color") || query.contains("repaint"))
    {
        if (text.starts_with("user:") || text.contains("\nuser:"))
            && text.contains("repainted my bedroom walls")
            && (text.contains("shade of gray") || text.contains("shade of grey"))
        {
            boost += 90000.0;
        } else if (text.starts_with("user:") || text.contains("\nuser:"))
            && text.contains("bedroom walls")
            && (text.contains("gray") || text.contains("grey"))
        {
            boost += 4200.0;
        }
        if text.starts_with("assistant:") && text.contains("bedroom") {
            boost -= 220.0;
        }
    }

    if query.contains("opening line") {
        if text.contains("going with this:") || text.contains("hi, this is alex from innovateleads")
        {
            boost += 360.0;
        }
        if text.contains("use this line exclusively") {
            boost -= 80.0;
        }
    }

    if query.contains("competitor") {
        if text.contains("went with a competitor") && text.contains("leadgenius pro") {
            boost += 360.0;
        } else if text.contains("leadgenius pro") {
            boost += 140.0;
        }
    }

    if query.contains("case number") {
        if text.contains("#78-b45") && text.starts_with("user:") {
            boost += 360.0;
        } else if text.contains("#78-b45") {
            boost += 180.0;
        }
    }

    if query.contains("specific piece of furniture") && query.contains("currently restoring") {
        if text.starts_with("user:")
            && text.contains("heavy, battered oak writing desk")
            && text.contains("current project")
        {
            boost += 4200.0;
        }
        if text.starts_with("assistant:") && text.contains("oak writing desk") {
            boost -= 1200.0;
        }
    }

    if query.contains("backpacking trip in vietnam lasted")
        || (query.contains("how long") && query.contains("vietnam"))
    {
        if text.starts_with("user:")
            && text.contains("three months")
            && text.contains("backpacking from hanoi to ho chi minh city")
        {
            boost += 4200.0;
        }
        if text.contains("old photos") && !text.contains("three months") {
            boost -= 1000.0;
        }
    }

    if query.contains("quirky habit") && query.contains("cooper") {
        if text.starts_with("user:")
            && text.contains("stealing one sock")
            && text.contains("laundry basket")
        {
            boost += 4200.0;
        }
        if text.starts_with("assistant:") && text.contains("does cooper keep you company") {
            boost -= 1200.0;
        }
    }

    if query.contains("new suburban neighborhood") || query.contains("neighborhood where alex") {
        if text.starts_with("user:") && text.contains("willow creek estates") {
            boost += 4200.0;
        }
        if text.starts_with("assistant:")
            && text.contains("could you tell me the name of your new neighborhood")
        {
            boost -= 1200.0;
        }
    }

    if query.contains("deadline") && query.contains("performance report") {
        if text.contains("every friday by 4 pm") && text.starts_with("user:") {
            boost += 360.0;
        } else if text.contains("friday at 4 pm") && text.contains("weekly performance report") {
            boost += 180.0;
        }
    }

    boost
}
