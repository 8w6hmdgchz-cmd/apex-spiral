use std::collections::VecDeque;

#[derive(Debug, Clone)]
struct Trace {
    id: String,
    importance: f64,
    novelty: f64,
    future_utility: f64,
    recurrence: f64,
    user_intent: f64,
    sensitivity_risk: f64,
}

impl Trace {
    fn score(&self) -> f64 {
        0.35 * self.importance
            + 0.20 * self.novelty
            + 0.20 * self.future_utility
            + 0.15 * self.recurrence
            + 0.10 * self.user_intent
            - 0.40 * self.sensitivity_risk
    }

    fn decision(&self) -> &'static str {
        let s = self.score();
        if self.sensitivity_risk >= 0.85 {
            "reject_sensitive"
        } else if s >= 0.85 {
            "promote_candidate"
        } else if s >= 0.70 {
            "ring_buffer"
        } else {
            "drop"
        }
    }
}

struct RingBuffer<T> {
    cap: usize,
    data: VecDeque<T>,
}

impl<T> RingBuffer<T> {
    fn new(cap: usize) -> Self {
        Self { cap, data: VecDeque::with_capacity(cap) }
    }

    fn push(&mut self, value: T) {
        if self.cap == 0 { return; }
        if self.data.len() == self.cap {
            self.data.pop_front();
        }
        self.data.push_back(value);
    }

    fn len(&self) -> usize { self.data.len() }
}

fn main() {
    let traces = vec![
        Trace { id: "a2a_auto_trigger_configured".into(), importance: 0.9, novelty: 0.7, future_utility: 0.9, recurrence: 0.6, user_intent: 0.8, sensitivity_risk: 0.0 },
        Trace { id: "one_off_chatter".into(), importance: 0.2, novelty: 0.1, future_utility: 0.1, recurrence: 0.0, user_intent: 0.0, sensitivity_risk: 0.0 },
        Trace { id: "raw_token_secret".into(), importance: 0.9, novelty: 0.9, future_utility: 0.7, recurrence: 0.4, user_intent: 0.2, sensitivity_risk: 1.0 },
    ];

    let mut ring = RingBuffer::new(8);
    for t in traces {
        let decision = t.decision();
        println!("trace={} score={:.3} decision={}", t.id, t.score(), decision);
        if decision == "ring_buffer" || decision == "promote_candidate" {
            ring.push(t);
        }
    }
    println!("ring_len={}", ring.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_overwrites_oldest() {
        let mut r = RingBuffer::new(2);
        r.push(1);
        r.push(2);
        r.push(3);
        assert_eq!(r.len(), 2);
        assert_eq!(r.data[0], 2);
        assert_eq!(r.data[1], 3);
    }

    #[test]
    fn sensitive_rejected() {
        let t = Trace { id: "s".into(), importance: 1.0, novelty: 1.0, future_utility: 1.0, recurrence: 1.0, user_intent: 1.0, sensitivity_risk: 1.0 };
        assert_eq!(t.decision(), "reject_sensitive");
    }

    #[test]
    fn useful_promotes() {
        let t = Trace { id: "u".into(), importance: 1.0, novelty: 0.9, future_utility: 1.0, recurrence: 0.8, user_intent: 0.8, sensitivity_risk: 0.0 };
        assert_eq!(t.decision(), "promote_candidate");
    }
}
