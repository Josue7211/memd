use std::collections::BTreeMap;

/// Porter-style stemmer (simplified English suffix stripping).
/// Matches LoCoMo paper's stemming normalization.
pub(crate) fn stem_token(word: &str) -> String {
    let w = word.to_ascii_lowercase();
    if let Some(base) = w.strip_suffix("ies") {
        return format!("{base}i");
    }
    if let Some(base) = w.strip_suffix("sses") {
        return format!("{base}ss");
    }
    if let Some(base) = w.strip_suffix("ness") {
        return base.to_string();
    }
    if w.ends_with('s') && !w.ends_with("ss") && w.len() > 3 {
        return w[..w.len() - 1].to_string();
    }
    if let Some(base) = w.strip_suffix("eed") {
        if base.len() > 1 {
            return format!("{base}ee");
        }
    }
    if let Some(base) = w.strip_suffix("ing") {
        if base.len() > 2 {
            return reduce_doubled_consonant(base);
        }
    }
    if let Some(base) = w.strip_suffix("ed") {
        if base.len() > 2 {
            return reduce_doubled_consonant(base);
        }
    }
    w
}

/// Reduce trailing doubled consonant (e.g. "runn" → "run", "hopp" → "hop").
fn reduce_doubled_consonant(word: &str) -> String {
    let bytes = word.as_bytes();
    if bytes.len() >= 2 {
        let last = bytes[bytes.len() - 1];
        let second_last = bytes[bytes.len() - 2];
        if last == second_last && !b"aeiou".contains(&last) {
            return word[..word.len() - 1].to_string();
        }
    }
    word.to_string()
}

/// Tokenize and stem a string for F1 computation.
/// Matches LoCoMo paper: lowercase, split on whitespace+punctuation, stem.
pub(crate) fn tokenize_and_stem(text: &str) -> Vec<String> {
    text.to_ascii_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| stem_token(t))
        .collect()
}

/// Token-level F1 score with stemming (LoCoMo paper protocol).
/// Uses frequency-aware multiset matching, not set-based.
pub(crate) fn token_f1(prediction: &str, gold: &str) -> f64 {
    let pred_tokens = tokenize_and_stem(prediction);
    let gold_tokens = tokenize_and_stem(gold);
    if pred_tokens.is_empty() || gold_tokens.is_empty() {
        return 0.0;
    }
    let mut pred_counts: BTreeMap<&str, usize> = BTreeMap::new();
    for t in &pred_tokens {
        *pred_counts.entry(t.as_str()).or_insert(0) += 1;
    }
    let mut gold_counts: BTreeMap<&str, usize> = BTreeMap::new();
    for t in &gold_tokens {
        *gold_counts.entry(t.as_str()).or_insert(0) += 1;
    }
    let intersection: usize = pred_counts
        .iter()
        .map(|(token, count)| count.min(gold_counts.get(token).unwrap_or(&0)))
        .sum();
    let precision = intersection as f64 / pred_tokens.len() as f64;
    let recall = intersection as f64 / gold_tokens.len() as f64;
    if precision + recall == 0.0 {
        return 0.0;
    }
    2.0 * precision * recall / (precision + recall)
}

/// LoCoMo adversarial category: check if model correctly abstains.
pub(crate) fn locomo_adversarial_check(prediction: &str) -> bool {
    let lower = prediction.to_ascii_lowercase();
    lower.contains("no information")
        || lower.contains("not mentioned")
        || lower.contains("cannot answer")
        || lower.contains("not available")
        || lower.contains("no relevant")
        || lower.contains("don't have")
        || lower.contains("do not have")
        || lower.contains("unanswerable")
}

/// Multiple-choice accuracy: extract choice letter from LLM response,
/// compare against ground_truth.
pub(crate) fn mc_accuracy(response: &str, ground_truth: &str) -> bool {
    let gt = ground_truth.trim().to_ascii_uppercase();
    let resp = response.trim().to_ascii_uppercase();
    if resp == gt {
        return true;
    }
    for pattern in ["ANSWER IS ", "ANSWER: ", "CHOICE: ", "OPTION: "] {
        if let Some(after) = resp.find(pattern) {
            let rest = &resp[after + pattern.len()..];
            if let Some(letter) = rest.chars().next() {
                if letter.to_string() == gt {
                    return true;
                }
            }
        }
    }
    if gt.len() == 1 {
        let gt_char = gt.chars().next().unwrap();
        if resp.starts_with(gt_char)
            && resp
                .get(1..2)
                .map_or(true, |c| !c.chars().next().unwrap().is_alphanumeric())
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f1_exact_match() {
        let score = token_f1("the cat sat on the mat", "the cat sat on the mat");
        assert!((score - 1.0).abs() < 0.001);
    }

    #[test]
    fn f1_partial_match() {
        let score = token_f1("the cat sat", "the cat sat on the mat");
        assert!((score - 0.667).abs() < 0.01);
    }

    #[test]
    fn f1_no_match() {
        let score = token_f1("dog runs fast", "the cat sat on the mat");
        assert!((score - 0.0).abs() < 0.001);
    }

    #[test]
    fn f1_empty_prediction() {
        let score = token_f1("", "the cat sat on the mat");
        assert!((score - 0.0).abs() < 0.001);
    }

    #[test]
    fn mc_exact_match() {
        assert!(mc_accuracy("A", "A"));
    }

    #[test]
    fn mc_no_match() {
        assert!(!mc_accuracy("A", "B"));
    }

    #[test]
    fn mc_extracts_from_text() {
        assert!(mc_accuracy("The answer is B", "B"));
    }

    #[test]
    fn stem_basic() {
        assert_eq!(stem_token("running"), "run");
        assert_eq!(stem_token("cats"), "cat");
    }
}
