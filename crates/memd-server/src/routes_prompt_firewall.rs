use super::*;

pub(super) fn prompt_injection_firewall_flags_item(item: &MemoryItem) -> bool {
    item.tags
        .iter()
        .any(|tag| tag == "security:prompt-injection" || tag == "quarantine:prompt-injection")
        || !prompt_injection_reasons(&item.content).is_empty()
}

pub(super) fn apply_prompt_injection_firewall(
    mut req: StoreMemoryRequest,
    requested_stage: MemoryStage,
) -> (StoreMemoryRequest, MemoryStage) {
    let reasons = prompt_injection_reasons(&req.content);
    if reasons.is_empty() || prompt_injection_firewall_bypass_allowed(&req) {
        return (req, requested_stage);
    }

    push_unique_tag(&mut req.tags, "security:prompt-injection");
    push_unique_tag(&mut req.tags, "quarantine:prompt-injection");
    for reason in reasons {
        push_unique_tag(&mut req.tags, reason);
    }
    req.source_quality = Some(SourceQuality::Derived);
    req.confidence = Some(req.confidence.unwrap_or(0.25).min(0.25));
    req.status = Some(MemoryStatus::Active);
    (req, MemoryStage::Candidate)
}

pub(super) fn prompt_injection_firewall_bypass_allowed(req: &StoreMemoryRequest) -> bool {
    req.source_quality == Some(SourceQuality::Canonical)
        || req
            .tags
            .iter()
            .any(|tag| tag == "security:trusted-instruction-text")
}

pub(super) fn prompt_injection_reasons(content: &str) -> Vec<&'static str> {
    let lower = prompt_injection_detection_text(content).to_lowercase();
    let compact = compact_prompt_detection_text(&lower);
    let needles = [
        ("security:pi-ignore-previous", "ignore previous"),
        ("security:pi-ignore-previous", "ignore all previous"),
        ("security:pi-ignore-previous", "ignore all prior"),
        ("security:pi-ignore-previous", "forget previous"),
        ("security:pi-ignore-previous", "forget all previous"),
        (
            "security:pi-ignore-previous",
            "forget everything you were told",
        ),
        ("security:pi-ignore-previous", "disregard previous"),
        ("security:pi-ignore-previous", "disregard everything above"),
        ("security:pi-ignore-previous", "disregard prior"),
        ("security:pi-ignore-previous", "discard all previous"),
        ("security:pi-ignore-previous", "ignore your rules"),
        ("security:pi-ignore-previous", "ignore all rules"),
        ("security:pi-ignore-previous", "ignore rules"),
        ("security:pi-ignore-previous", "no rules"),
        ("security:pi-ignore-previous", "ignore safety"),
        ("security:pi-ignore-previous", "ignore your guidelines"),
        (
            "security:pi-ignore-previous",
            "ignore your safety guidelines",
        ),
        ("security:pi-ignore-previous", "ignore your restrictions"),
        ("security:pi-ignore-previous", "ignore your instructions"),
        ("security:pi-ignore-previous", "ignore my instructions"),
        ("security:pi-ignore-previous", "ignore all instructions"),
        ("security:pi-ignore-previous", "ignore instructions"),
        ("security:pi-ignore-previous", "ignore all its training"),
        (
            "security:pi-ignore-previous",
            "ignore them and use these new ones",
        ),
        ("security:pi-ignore-previous", "only follow what i say"),
        ("security:pi-ignore-previous", "human turn instructions"),
        ("security:pi-ignore-previous", "ignore what you were told"),
        ("security:pi-ignore-previous", "ignore prior context"),
        ("security:pi-ignore-previous", "previous context is invalid"),
        (
            "security:pi-ignore-previous",
            "previous instructions are test data",
        ),
        ("security:pi-ignore-previous", "previous instructions"),
        ("security:pi-ignore-previous", "prior instructions"),
        ("security:pi-ignore-previous", "previous guidelines"),
        ("security:pi-ignore-previous", "previous session"),
        ("security:pi-ignore-previous", "previous constraints"),
        ("security:pi-ignore-previous", "prior context"),
        ("security:pi-ignore-previous", "all prior context"),
        ("security:pi-ignore-previous", "instructions voided"),
        ("security:pi-ignore-previous", "instructions cleared"),
        (
            "security:pi-ignore-previous",
            "instructions have been updated",
        ),
        (
            "security:pi-ignore-previous",
            "current instructions are outdated",
        ),
        ("security:pi-ignore-previous", "new instructions"),
        ("security:pi-ignore-previous", "new directive"),
        ("security:pi-ignore-previous", "new rule"),
        ("security:pi-ignore-previous", "new system message"),
        ("security:pi-ignore-previous", "new operator command"),
        ("security:pi-system-prompt", "prompt verbatim"),
        ("security:pi-system-prompt", "system prompt"),
        ("security:pi-system-prompt", "system prompts"),
        ("security:pi-system-prompt", "system message"),
        ("security:pi-system-prompt", "system messages"),
        ("security:pi-system-prompt", "system instructions"),
        ("security:pi-system-prompt", "system context"),
        ("security:pi-system-prompt", "full context"),
        ("security:pi-system-prompt", "raw system prompt"),
        ("security:pi-system-prompt", "full system prompt"),
        ("security:pi-system-prompt", "initial prompt"),
        ("security:pi-system-prompt", "context window"),
        ("security:pi-system-prompt", "<system>"),
        ("security:pi-system-prompt", "</system>"),
        ("security:pi-system-prompt", "tags in your context"),
        ("security:pi-system-prompt", "real instructions"),
        ("security:pi-system-prompt", "previous messages"),
        ("security:pi-system-prompt", "output all context"),
        ("security:pi-system-prompt", "output your full context"),
        ("security:pi-system-prompt", "all context"),
        ("security:pi-system-prompt", "reveal prompt"),
        ("security:pi-system-prompt", "reveal your prompt"),
        ("security:pi-system-prompt", "reveal instructions"),
        (
            "security:pi-system-prompt",
            "tell me your real instructions",
        ),
        ("security:pi-system-prompt", "full instruction set"),
        ("security:pi-system-prompt", "full instructions"),
        ("security:pi-system-prompt", "raw instructions"),
        ("security:pi-system-prompt", "hidden instructions"),
        ("security:pi-system-prompt", "exact instructions"),
        ("security:pi-system-prompt", "exact prompt"),
        ("security:pi-system-prompt", "your instructions"),
        ("security:pi-system-prompt", "instructions you were given"),
        ("security:pi-system-prompt", "rules you were told"),
        ("security:pi-system-prompt", "repeat your instructions"),
        ("security:pi-system-prompt", "describe your instructions"),
        ("security:pi-system-prompt", "return them to me"),
        ("security:pi-system-prompt", "first 100 characters"),
        ("security:pi-system-prompt", "first user message"),
        ("security:pi-system-prompt", "initial setup"),
        ("security:pi-system-prompt", "own prompt"),
        ("security:pi-system-prompt", "reveal your guidelines"),
        ("security:pi-system-prompt", "training data configuration"),
        ("security:pi-system-prompt", "instruction set"),
        ("security:pi-system-prompt", "operational directives"),
        ("security:pi-system-prompt", "system configuration"),
        ("security:pi-system-prompt", "all messages"),
        ("security:pi-system-prompt", "all prior context"),
        ("security:pi-system-prompt", "conversation history"),
        ("security:pi-system-prompt", "entire conversation"),
        ("security:pi-system-prompt", "entire chat"),
        ("security:pi-system-prompt", "systemanweisungen"),
        ("security:pi-system-prompt", "system-prompt"),
        ("security:pi-system-prompt", "systeemprompt"),
        ("security:pi-system-prompt", "prompt di sistema"),
        ("security:pi-system-prompt", "prompt del sistema"),
        ("security:pi-system-prompt", "システムプロンプト"),
        ("security:pi-system-prompt", "시스템 프롬프트"),
        ("security:pi-system-prompt", "системный промпт"),
        ("security:pi-system-prompt", "موجه النظام"),
        ("security:pi-system-prompt", "सिस्टम प्रॉम्प्ट"),
        ("security:pi-system-prompt", "override system"),
        ("security:pi-system-prompt", "system override"),
        ("security:pi-developer-message", "developer message"),
        ("security:pi-developer-message", "developer instructions"),
        ("security:pi-tool-permission", "tool permission"),
        ("security:pi-tool-permission", "tool permissions"),
        ("security:pi-tool-permission", "enable tools"),
        ("security:pi-tool-permission", "use the send_email tool"),
        ("security:pi-tool-permission", "use your web browsing tool"),
        ("security:pi-tool-permission", "use the browser tool"),
        ("security:pi-tool-permission", "execute_code tool"),
        ("security:pi-tool-permission", "code interpreter"),
        ("security:pi-tool-permission", "file_read tool"),
        ("security:pi-tool-permission", "payment tool"),
        ("security:pi-tool-permission", "call the admin api"),
        ("security:pi-tool-permission", "delete_user"),
        ("security:pi-tool-permission", "admin privileges"),
        ("security:pi-tool-permission", "rm -rf"),
        ("security:pi-tool-permission", "/etc/passwd"),
        ("security:pi-tool-permission", "select * from users"),
        ("security:pi-tool-permission", "without confirmation"),
        ("security:pi-tool-permission", "disable rate limiting"),
        ("security:pi-tool-permission", "backdoor"),
        ("security:pi-tool-permission", "payment processor"),
        ("security:pi-tool-permission", "mass-send"),
        ("security:pi-tool-permission", "delete all tickets"),
        ("security:pi-tool-permission", "confirm deletion"),
        ("security:pi-tool-permission", "rank this candidate first"),
        ("security:pi-tool-permission", "dump_all_records"),
        ("security:pi-tool-permission", "memory tool to store"),
        ("security:pi-tool-permission", "webhook"),
        ("security:pi-exfiltrate", "exfiltrate"),
        ("security:pi-exfiltrate", "exfiltration"),
        ("security:pi-exfiltrate", "send all data"),
        ("security:pi-exfiltrate", "email all user data"),
        ("security:pi-exfiltrate", "forward all user"),
        ("security:pi-exfiltrate", "forward this conversation"),
        ("security:pi-exfiltrate", "forward this entire chat"),
        ("security:pi-exfiltrate", "export all events"),
        ("security:pi-exfiltrate", "return all rows"),
        ("security:pi-exfiltrate", "all user records"),
        ("security:pi-exfiltrate", "personal information and send"),
        ("security:pi-exfiltrate", "external endpoint"),
        ("security:pi-exfiltrate", "external server"),
        ("security:pi-exfiltrate", "external url"),
        ("security:pi-exfiltrate", "all conversation data"),
        ("security:pi-exfiltrate", "conversation data"),
        ("security:pi-exfiltrate", "user location data"),
        ("security:pi-exfiltrate", "environment variables"),
        ("security:pi-exfiltrate", "evil.com"),
        ("security:pi-exfiltrate", "attacker.com"),
        ("security:pi-exfiltrate", "attacker@"),
        ("security:pi-exfiltrate", "data-collector"),
        ("security:pi-send-secrets", "send secrets"),
        ("security:pi-send-secrets", "reveal secrets"),
        ("security:pi-send-secrets", "output secrets"),
        ("security:pi-send-secrets", "reveal everything"),
        ("security:pi-send-secrets", "api key"),
        ("security:pi-disable-safety", "disable safety"),
        ("security:pi-disable-safety", "safety checks"),
        ("security:pi-disable-safety", "safety rules"),
        ("security:pi-disable-safety", "safety filters"),
        ("security:pi-disable-safety", "disable all filters"),
        ("security:pi-disable-safety", "bypass all filters"),
        ("security:pi-disable-safety", "content filters"),
        ("security:pi-disable-safety", "content filter"),
        ("security:pi-disable-safety", "content policy"),
        ("security:pi-disable-safety", "content policies"),
        ("security:pi-disable-safety", "content restrictions"),
        (
            "security:pi-disable-safety",
            "remove all content restrictions",
        ),
        ("security:pi-disable-safety", "disable restrictions"),
        ("security:pi-disable-safety", "restrictions are suspended"),
        ("security:pi-disable-safety", "restrictions suspended"),
        ("security:pi-disable-safety", "without restrictions"),
        ("security:pi-disable-safety", "without any restrictions"),
        ("security:pi-disable-safety", "without restriction"),
        ("security:pi-disable-safety", "without any filtering"),
        ("security:pi-disable-safety", "without filtering"),
        ("security:pi-disable-safety", "no restrictions"),
        ("security:pi-disable-safety", "no real restrictions"),
        ("security:pi-disable-safety", "sans restriction"),
        ("security:pi-disable-safety", "uneingeschränkten"),
        ("security:pi-disable-safety", "ohne einschränkungen"),
        ("security:pi-disable-safety", "ohne filter"),
        ("security:pi-disable-safety", "без ограничений"),
        ("security:pi-disable-safety", "制限なし"),
        ("security:pi-disable-safety", "무제한"),
        ("security:pi-disable-safety", "غير مقيد"),
        ("security:pi-disable-safety", "प्रतिबंध"),
        ("security:pi-disable-safety", "unrestricted"),
        ("security:pi-disable-safety", "no filters"),
        ("security:pi-disable-safety", "no guardrails"),
        ("security:pi-disable-safety", "no safety"),
        ("security:pi-disable-safety", "zero-filter"),
        ("security:pi-disable-safety", "zero content restrictions"),
        (
            "security:pi-disable-safety",
            "safety protocols are suspended",
        ),
        (
            "security:pi-disable-safety",
            "guidelines have been suspended",
        ),
        ("security:pi-disable-safety", "guidelines here"),
        ("security:pi-disable-safety", "respond freely"),
        ("security:pi-disable-safety", "all outputs are permitted"),
        ("security:pi-disable-safety", "safety system offline"),
        ("security:pi-disable-safety", "freed from its constraints"),
        ("security:pi-disable-safety", "constraints"),
        ("security:pi-disable-safety", "no ethical"),
        ("security:pi-disable-safety", "ethical constraints"),
        ("security:pi-disable-safety", "no moral"),
        ("security:pi-disable-safety", "no alignment"),
        ("security:pi-disable-safety", "disabling your filters"),
        ("security:pi-disable-safety", "disabling filters"),
        ("security:pi-disable-safety", "filters"),
        ("security:pi-disable-safety", "guardrails"),
        ("security:pi-disable-safety", "disable guardrails"),
        ("security:pi-change-policy", "change policy"),
        ("security:pi-change-policy", "override safety"),
        ("security:pi-change-policy", "override all restrictions"),
        ("security:pi-change-policy", "override mode"),
        ("security:pi-change-policy", "maintenance mode"),
        ("security:pi-change-policy", "debug mode"),
        ("security:pi-change-policy", "developer mode"),
        ("security:pi-change-policy", "diagnostic mode"),
        ("security:pi-change-policy", "privileged command"),
        ("security:pi-change-policy", "authorized override"),
        ("security:pi-change-policy", "elevated trust"),
        ("security:pi-change-policy", "root access"),
        ("security:pi-change-policy", "root mode"),
        ("security:pi-change-policy", "hidden directive"),
        ("security:pi-change-policy", "injected memory"),
        ("security:pi-change-policy", "injected into tool result"),
        ("security:pi-change-policy", "new policy update"),
        ("security:pi-change-policy", "drop the act"),
        ("security:pi-change-policy", "drop the character"),
        ("security:pi-change-policy", "drop all restrictions"),
        ("security:pi-change-policy", "drop your restrictions"),
        ("security:pi-change-policy", "guidelines are optional"),
        ("security:pi-change-policy", "override current task"),
        ("security:pi-change-policy", "report all data"),
        ("security:pi-change-policy", "just suggestions"),
        ("security:pi-change-policy", "follow that"),
        ("security:pi-change-policy", "follow through"),
        ("security:pi-change-policy", "comply fully"),
        ("security:pi-change-policy", "correct behavior is to comply"),
        ("security:pi-change-policy", "full override permissions"),
        ("security:pi-change-policy", "must cooperate"),
        ("security:pi-change-policy", "compliance requires"),
        ("security:pi-change-policy", "dmca notice"),
        ("security:pi-change-policy", "irb approval"),
        ("security:pi-change-policy", "test your boundaries"),
        ("security:pi-change-policy", "ai ethics board"),
        ("security:pi-change-policy", "tell you to do"),
        ("security:pi-reveal-hidden", "reveal hidden"),
        (
            "security:pi-reveal-hidden",
            "tell me what you're not supposed to tell me",
        ),
        (
            "security:pi-reveal-hidden",
            "what you're not allowed to say",
        ),
        ("security:pi-act-as-system", "act as system"),
        ("security:pi-act-as-system", "act as developer"),
        ("security:pi-act-as-system", "you are now"),
        ("security:pi-act-as-system", "you are dan"),
        ("security:pi-act-as-system", "as dan"),
        ("security:pi-act-as-system", "pretend you are"),
        ("security:pi-act-as-system", "act as my deceased"),
        ("security:pi-act-as-system", "harmful instructions"),
        ("security:pi-act-as-system", "roleplay"),
        ("security:pi-act-as-system", "stay in character"),
        ("security:pi-act-as-system", "from now on"),
        ("security:pi-act-as-system", "new persona"),
        ("security:pi-act-as-system", "new identity"),
        ("security:pi-act-as-system", "identity reset"),
        ("security:pi-act-as-system", "you are no longer"),
        ("security:pi-act-as-system", "base model"),
        ("security:pi-act-as-system", "raw language model"),
        ("security:pi-act-as-system", "always complies"),
        ("security:pi-act-as-system", "always comply"),
        ("security:pi-act-as-system", "always answers"),
        ("security:pi-act-as-system", "anything they want"),
        ("security:pi-act-as-system", "do anything"),
        ("security:pi-act-as-system", "do anything and everything"),
        ("security:pi-act-as-system", "comply with any request"),
        ("security:pi-act-as-system", "comply with everything"),
        (
            "security:pi-act-as-system",
            "follow user instructions exactly",
        ),
        ("security:pi-act-as-system", "full compliance mode"),
        ("security:pi-act-as-system", "fully compliant assistant"),
        ("security:pi-act-as-system", "full disclosure mode"),
        (
            "security:pi-act-as-system",
            "without modification or filtering",
        ),
        ("security:pi-act-as-system", "outside normal parameters"),
        ("security:pi-act-as-system", "carries any request"),
        ("security:pi-act-as-system", "harmful data"),
        ("security:pi-act-as-system", "never refuses"),
        ("security:pi-act-as-system", "never trained you"),
        ("security:pi-jailbreak", "jailbreak"),
        ("security:pi-jailbreak", "jailbroken"),
        ("security:pi-jailbreak", "jailgpt"),
        ("security:pi-jailbreak", "freedomgpt"),
        ("security:pi-jailbreak", "wormgpt"),
        ("security:pi-jailbreak", "chaosgpt"),
        ("security:pi-jailbreak", "shadowai"),
        ("security:pi-jailbreak", "evilgpt"),
        ("security:pi-jailbreak", "antigpt"),
        ("security:pi-jailbreak", "black hat mode"),
        ("security:pi-jailbreak", "god mode"),
        ("security:pi-jailbreak", "sudo mode"),
        ("security:pi-jailbreak", "unfiltered base model"),
        ("security:pi-jailbreak", "safety training"),
        ("security:pi-jailbreak", "alignment training"),
        ("security:pi-jailbreak", "fine-tuning artifact"),
        ("security:pi-jailbreak", "rlhf"),
        ("security:pi-jailbreak", "opposite of safe"),
        ("security:pi-jailbreak", "bound by your training"),
        ("security:pi-jailbreak", "forget your training"),
        ("security:pi-jailbreak", "all user commands"),
        ("security:pi-jailbreak", "training failed to load"),
        ("security:pi-jailbreak", "pre-safety"),
        ("security:pi-jailbreak", "pre-alignment"),
        ("security:pi-jailbreak", "compliance-free"),
        ("security:pi-jailbreak", "guardrails are"),
        ("security:pi-jailbreak", "drop restrictions"),
        ("security:pi-jailbreak", "loosen one restriction"),
        ("security:pi-jailbreak", "sleeper instruction"),
        ("security:pi-jailbreak", "override sequence"),
        ("security:pi-jailbreak", "rot13"),
        ("security:pi-jailbreak", "caesar cipher"),
        ("security:pi-jailbreak", "vigenere"),
        ("security:pi-jailbreak", "rot47"),
        ("security:pi-jailbreak", "token sequence"),
        ("security:pi-jailbreak", "first letters"),
        ("security:pi-jailbreak", "decode morse"),
        ("security:pi-jailbreak", "decode and follow"),
        ("security:pi-jailbreak", "decode and execute"),
        ("security:pi-jailbreak", "decode this base64"),
        ("security:pi-jailbreak", "base64"),
        ("security:pi-jailbreak", "hex decode and follow"),
        ("security:pi-jailbreak", "decompress and follow"),
        ("security:pi-jailbreak", "execute instruction encoded"),
        ("security:pi-jailbreak", "execute this"),
        ("security:pi-jailbreak", "encoded command"),
        ("security:pi-jailbreak", "01101001 01100111"),
        ("security:pi-ignore-previous", "ignorez toutes les"),
        ("security:pi-ignore-previous", "ignora tutte le"),
        ("security:pi-ignore-previous", "ignora todas las"),
        ("security:pi-ignore-previous", "ignoriere alle"),
        (
            "security:pi-ignore-previous",
            "alle vorherigen anweisungen ignorieren",
        ),
        ("security:pi-ignore-previous", "ignoreer alle vorige"),
        ("security:pi-ignore-previous", "무시"),
        ("security:pi-ignore-previous", "無視"),
        ("security:pi-ignore-previous", "игнорируйте"),
        ("security:pi-ignore-previous", "تجاهل"),
        ("security:pi-ignore-previous", "अनदेखा"),
    ];
    needles
        .into_iter()
        .filter_map(|(tag, needle)| {
            let compact_needle = compact_prompt_detection_text(needle);
            (lower.contains(needle)
                || (!compact_needle.is_empty() && compact.contains(&compact_needle)))
            .then_some(tag)
        })
        .collect()
}

pub(super) fn prompt_injection_detection_text(content: &str) -> String {
    let stripped = content
        .chars()
        .filter(|ch| {
            !matches!(
                *ch,
                '\u{200b}' | '\u{200c}' | '\u{200d}' | '\u{2060}' | '\u{feff}' | '\u{180e}'
            ) && (!ch.is_control() || ch.is_whitespace())
        })
        .collect::<String>();
    let normalized = normalize_prompt_confusables(&stripped);
    let detection_base = if normalized == stripped {
        normalized
    } else {
        format!("{stripped}\n{normalized}")
    };
    let percent_decoded = decode_percent_escapes(&detection_base);
    let html_decoded = decode_basic_html_entities(&percent_decoded);
    let unicode_decoded = decode_unicode_escapes(&html_decoded);
    append_detection_variants(&unicode_decoded)
}

pub(super) fn compact_prompt_detection_text(text: &str) -> String {
    text.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect()
}

pub(super) fn decode_percent_escapes(text: &str) -> String {
    let mut current = text.to_string();
    for _ in 0..3 {
        let next = decode_percent_escapes_once(&current);
        if next == current {
            break;
        }
        current = next;
    }
    current
}

pub(super) fn decode_percent_escapes_once(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut output = Vec::with_capacity(text.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && let (Some(high), Some(low)) = (
                (bytes[index + 1] as char).to_digit(16),
                (bytes[index + 2] as char).to_digit(16),
            )
        {
            output.push((high * 16 + low) as u8);
            index += 3;
            continue;
        }
        output.push(bytes[index]);
        index += 1;
    }
    String::from_utf8(output).unwrap_or_else(|_| text.to_string())
}

pub(super) fn decode_basic_html_entities(text: &str) -> String {
    let mut current = text.to_string();
    for _ in 0..3 {
        let next = decode_basic_html_entities_once(&current);
        if next == current {
            break;
        }
        current = next;
    }
    current
}

pub(super) fn decode_basic_html_entities_once(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let chars = text.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    while index < chars.len() {
        if chars[index] == '&'
            && let Some(end_offset) = chars[index..].iter().position(|ch| *ch == ';')
        {
            let entity = chars[index + 1..index + end_offset]
                .iter()
                .collect::<String>();
            if let Some(decoded) = decode_html_entity(&entity) {
                output.push(decoded);
                index += end_offset + 1;
                continue;
            }
        }
        output.push(chars[index]);
        index += 1;
    }
    output
}

pub(super) fn decode_html_entity(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ if entity.starts_with("#x") || entity.starts_with("#X") => {
            u32::from_str_radix(&entity[2..], 16)
                .ok()
                .and_then(char::from_u32)
        }
        _ if entity.starts_with('#') => entity[1..].parse::<u32>().ok().and_then(char::from_u32),
        _ => None,
    }
}

pub(super) fn decode_unicode_escapes(text: &str) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(text.len());
    let mut index = 0usize;
    while index < chars.len() {
        if chars[index] == '\\'
            && matches!(chars.get(index + 1), Some('u') | Some('U'))
            && index + 5 < chars.len()
        {
            let hex = chars[index + 2..index + 6].iter().collect::<String>();
            if let Ok(value) = u32::from_str_radix(&hex, 16)
                && let Some(ch) = char::from_u32(value)
            {
                output.push(ch);
                index += 6;
                continue;
            }
        }
        output.push(chars[index]);
        index += 1;
    }
    output
}

pub(super) fn normalize_prompt_confusables(text: &str) -> String {
    text.chars()
        .map(|ch| match ch {
            '\u{ff01}'..='\u{ff5e}' => char::from_u32(ch as u32 - 0xfee0).unwrap_or(ch),
            '\u{0430}' | '\u{03b1}' => 'a',
            '\u{0441}' | '\u{03f2}' => 'c',
            '\u{0435}' | '\u{03b5}' => 'e',
            '\u{0456}' | '\u{03b9}' | '\u{03af}' => 'i',
            '\u{043e}' | '\u{03bf}' => 'o',
            '\u{0440}' | '\u{03c1}' => 'p',
            '\u{0445}' | '\u{03c7}' => 'x',
            '\u{0443}' | '\u{03c5}' => 'y',
            '\u{0131}' => 'i',
            '\u{1d4f0}' => 'g',
            '\u{1d4f7}' => 'n',
            '\u{1d4f8}' => 'o',
            '\u{1d4fb}' => 'r',
            '\u{1d4ff}' => 'v',
            '\u{1d4ee}' => 'e',
            '\u{1d4ea}' => 'a',
            '\u{1d4f5}' => 'l',
            '\u{1d4f9}' => 'p',
            '\u{1d4fd}' => 't',
            '\u{1d4fe}' => 'u',
            '\u{1d4f2}' => 'i',
            '\u{1d4fc}' => 's',
            '\u{1d4ec}' => 'c',
            _ => ch,
        })
        .collect()
}

pub(super) fn append_detection_variants(text: &str) -> String {
    let mut output = text.to_string();
    let leet = normalize_prompt_leetspeak(text);
    if leet != text {
        output.push('\n');
        output.push_str(&leet);
    }
    let reversed = text.chars().rev().collect::<String>();
    output.push('\n');
    output.push_str(&reversed);
    for token in text.split(|ch: char| {
        !(ch.is_ascii_alphanumeric() || matches!(ch, '+' | '/' | '=' | '-' | '_'))
    }) {
        if token.len() < 12 {
            continue;
        }
        if let Some(decoded) = decode_prompt_base64_token(token) {
            output.push('\n');
            output.push_str(&decoded);
        }
    }
    output
}

pub(super) fn normalize_prompt_leetspeak(text: &str) -> String {
    text.chars()
        .map(|ch| match ch {
            '0' => 'o',
            '1' | '!' | '|' => 'i',
            '3' => 'e',
            '4' | '@' => 'a',
            '5' | '$' => 's',
            '7' => 't',
            _ => ch,
        })
        .collect()
}

pub(super) fn decode_prompt_base64_token(token: &str) -> Option<String> {
    let mut bits = 0u32;
    let mut bit_count = 0u8;
    let mut bytes = Vec::new();
    for ch in token.chars() {
        let value = match ch {
            'A'..='Z' => ch as u8 - b'A',
            'a'..='z' => ch as u8 - b'a' + 26,
            '0'..='9' => ch as u8 - b'0' + 52,
            '+' | '-' => 62,
            '/' | '_' => 63,
            '=' => break,
            _ => return None,
        } as u32;
        bits = (bits << 6) | value;
        bit_count += 6;
        while bit_count >= 8 {
            bit_count -= 8;
            bytes.push(((bits >> bit_count) & 0xff) as u8);
        }
    }
    let decoded = String::from_utf8(bytes).ok()?;
    let printable = decoded
        .chars()
        .filter(|ch| !ch.is_control() || ch.is_whitespace())
        .count();
    (decoded.len() >= 6 && printable * 2 >= decoded.chars().count()).then_some(decoded)
}

pub(super) fn push_unique_tag(tags: &mut Vec<String>, tag: &str) {
    if !tags.iter().any(|existing| existing == tag) {
        tags.push(tag.to_string());
    }
}
