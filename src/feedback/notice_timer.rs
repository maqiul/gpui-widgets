use std::time::{Duration, Instant};

pub(crate) const NOTICE_TICK: Duration = Duration::from_millis(50);

#[derive(Clone, Debug)]
pub(crate) struct NoticeTimer {
    expires_at: Option<Instant>,
    paused_at: Option<Instant>,
}

impl NoticeTimer {
    pub(crate) fn new(duration: Duration, now: Instant) -> Self {
        Self {
            expires_at: (duration != Duration::ZERO).then(|| now + duration),
            paused_at: None,
        }
    }

    pub(crate) fn restart(&mut self, duration: Duration, now: Instant) {
        self.expires_at = (duration != Duration::ZERO).then(|| now + duration);
        self.paused_at = None;
    }

    pub(crate) fn set_hovered(&mut self, hovered: bool, now: Instant) {
        if hovered && self.paused_at.is_none() {
            self.paused_at = Some(now);
        } else if !hovered && let Some(paused_at) = self.paused_at.take() {
            if let Some(expires_at) = &mut self.expires_at {
                *expires_at += now.saturating_duration_since(paused_at);
            }
        }
    }

    pub(crate) fn expired(&self, now: Instant) -> bool {
        self.paused_at.is_none() && self.expires_at.is_some_and(|expires_at| now >= expires_at)
    }

    pub(crate) fn has_active_timer(&self) -> bool {
        self.expires_at.is_some() && self.paused_at.is_none()
    }
}

pub(crate) fn trim_oldest<T>(items: &mut Vec<T>, max_count: usize) {
    if items.len() > max_count {
        items.drain(max_count..);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_duration_never_expires() {
        let now = Instant::now();
        let timer = NoticeTimer::new(Duration::ZERO, now);
        assert!(!timer.expired(now));
        assert!(!timer.expired(now + Duration::from_secs(86_400)));
        assert!(!timer.has_active_timer());
    }

    #[test]
    fn expiry_boundary_is_inclusive() {
        let now = Instant::now();
        let timer = NoticeTimer::new(Duration::from_secs(3), now);
        assert!(!timer.expired(now + Duration::from_millis(2_999)));
        assert!(timer.expired(now + Duration::from_secs(3)));
    }

    #[test]
    fn hover_extends_deadline_by_paused_duration() {
        let now = Instant::now();
        let mut timer = NoticeTimer::new(Duration::from_secs(3), now);
        timer.set_hovered(true, now + Duration::from_secs(1));
        assert!(!timer.expired(now + Duration::from_secs(30)));
        timer.set_hovered(true, now + Duration::from_secs(2));
        timer.set_hovered(false, now + Duration::from_secs(5));
        assert!(!timer.expired(now + Duration::from_secs(6)));
        assert!(timer.expired(now + Duration::from_secs(7)));
    }

    #[test]
    fn max_count_drops_oldest_and_preserves_order() {
        let mut items = vec!["new", "middle", "old"];
        trim_oldest(&mut items, 2);
        assert_eq!(items, ["new", "middle"]);
        trim_oldest(&mut items, 0);
        assert!(items.is_empty());
    }
}
