use std::collections::BTreeMap;

fn counts(labels: &[&str]) -> BTreeMap<String, usize> {
    let mut m = BTreeMap::new();
    for label in labels {
        *m.entry((*label).to_string()).or_insert(0) += 1;
    }
    m
}

fn gini(labels: &[&str]) -> f64 {
    if labels.is_empty() {
        return 0.0;
    }
    let n = labels.len() as f64;
    let sum_sq: f64 = counts(labels)
        .values()
        .map(|&c| {
            let p = c as f64 / n;
            p * p
        })
        .sum();
    1.0 - sum_sq
}

fn entropy(labels: &[&str]) -> f64 {
    if labels.is_empty() {
        return 0.0;
    }
    let n = labels.len() as f64;
    counts(labels)
        .values()
        .map(|&c| {
            let p = c as f64 / n;
            if p == 0.0 {
                0.0
            } else {
                -p * p.log2()
            }
        })
        .sum()
}

fn weighted_split_metric<F>(left: &[&str], right: &[&str], metric: F) -> f64
where
    F: Fn(&[&str]) -> f64,
{
    let n = (left.len() + right.len()) as f64;
    if n == 0.0 {
        return 0.0;
    }
    (left.len() as f64 / n) * metric(left) + (right.len() as f64 / n) * metric(right)
}

fn main() {
    // Demo labels: accept/reject/rewrite outcomes for candidate skill mutations.
    let parent = [
        "accept", "accept", "rewrite", "reject", "accept", "rewrite", "reject", "accept",
    ];
    let left = ["accept", "accept", "accept", "rewrite"];
    let right = ["reject", "rewrite", "reject", "accept"];

    let g_parent = gini(&parent);
    let g_gain = g_parent - weighted_split_metric(&left, &right, gini);
    let h_parent = entropy(&parent);
    let info_gain = h_parent - weighted_split_metric(&left, &right, entropy);

    println!("parent_gini={:.6}", g_parent);
    println!("gini_gain={:.6}", g_gain);
    println!("parent_entropy={:.6}", h_parent);
    println!("information_gain={:.6}", info_gain);

    let selected = if g_gain >= info_gain {
        "gini_path"
    } else {
        "entropy_path"
    };
    println!("selected_mutation_path={}", selected);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_gini_zero() {
        assert!((gini(&["a", "a", "a"]) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn balanced_binary_gini_half() {
        assert!((gini(&["a", "b"]) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn entropy_binary_one() {
        assert!((entropy(&["a", "b"]) - 1.0).abs() < 1e-9);
    }
}
