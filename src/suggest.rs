/// Damerau-Levenshtein-style "did you mean" suggestions for typos like
/// `fltapak` -> `flatpak`. Dependency-free and tiny by design.
pub fn suggest(input: &str, candidates: &[String]) -> Option<String> {
    suggest_str(
        input,
        &candidates.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
    )
}

pub fn suggest_str(input: &str, candidates: &[&str]) -> Option<String> {
    let input = input.to_ascii_lowercase();
    let mut best: Option<(u32, &str)> = None;
    for cand in candidates {
        let d = edit_distance(&input, &cand.to_ascii_lowercase());
        let threshold = if cand.len() <= 4 { 1 } else { 2 };
        if d <= threshold {
            match best {
                Some((bd, _)) if bd <= d => {}
                _ => best = Some((d, cand)),
            }
        }
    }
    best.map(|(_, c)| c.to_string())
}

/// Standard Levenshtein distance with early exit for short strings.
pub fn edit_distance(a: &str, b: &str) -> u32 {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len() as u32;
    }
    if b.is_empty() {
        return a.len() as u32;
    }

    let mut prev: Vec<u32> = (0..=b.len() as u32).collect();
    let mut curr = vec![0u32; b.len() + 1];

    for (i, ca) in a.iter().enumerate() {
        curr[0] = i as u32 + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr[j + 1] = (prev[j] + cost).min(prev[j + 1] + 1).min(curr[j] + 1);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn backends() -> Vec<String> {
        ["apt", "pacman", "flatpak", "snap", "dnf", "brew"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn suggests_for_typo() {
        assert_eq!(suggest("fltapak", &backends()), Some("flatpak".to_string()));
        assert_eq!(suggest("aptt", &backends()), Some("apt".to_string()));
        assert_eq!(suggest("snapp", &backends()), Some("snap".to_string()));
    }

    #[test]
    fn no_suggestion_for_nonsense() {
        assert_eq!(suggest("zzzzzzzz", &backends()), None);
    }

    #[test]
    fn distance_basics() {
        assert_eq!(edit_distance("", ""), 0);
        assert_eq!(edit_distance("abc", "abc"), 0);
        assert_eq!(edit_distance("abc", ""), 3);
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(edit_distance("fltapak", "flatpak"), 2);
    }
}
