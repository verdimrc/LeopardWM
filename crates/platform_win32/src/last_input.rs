//! The last key press and mouse click seen by the low-level hooks, so the
//! daemon can tell how a launcher was closed (Esc vs. Enter or a click).

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

static KEY_SEEN: AtomicBool = AtomicBool::new(false);
static KEY_VK: AtomicU32 = AtomicU32::new(0);
static KEY_TICK: AtomicU32 = AtomicU32::new(0);
static CLICK_SEEN: AtomicBool = AtomicBool::new(false);
static CLICK_TICK: AtomicU32 = AtomicU32::new(0);

/// The most recent of the last key press and the last mouse click.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LastInput {
    /// A key press, by virtual-key code.
    Key(u32),
    Click,
    None,
}

/// Record a key press; `tick` is the hook's `GetTickCount` timestamp.
pub(crate) fn record_key_down(vk: u32, tick: u32) {
    KEY_VK.store(vk, Ordering::Relaxed);
    KEY_TICK.store(tick, Ordering::Relaxed);
    KEY_SEEN.store(true, Ordering::Release);
}

/// Record a mouse button press; `tick` is the hook's `GetTickCount` timestamp.
pub(crate) fn record_click(tick: u32) {
    CLICK_TICK.store(tick, Ordering::Relaxed);
    CLICK_SEEN.store(true, Ordering::Release);
}

/// The latest key press or mouse click seen so far.
pub fn last_input() -> LastInput {
    let key = KEY_SEEN.load(Ordering::Acquire).then(|| {
        (
            KEY_VK.load(Ordering::Relaxed),
            KEY_TICK.load(Ordering::Relaxed),
        )
    });
    let click = CLICK_SEEN
        .load(Ordering::Acquire)
        .then(|| CLICK_TICK.load(Ordering::Relaxed));
    latest(key, click)
}

fn latest(key: Option<(u32, u32)>, click: Option<u32>) -> LastInput {
    match (key, click) {
        (Some((vk, key_tick)), Some(click_tick)) => {
            // Tick counts wrap; the later one is less than half the range ahead.
            if (click_tick.wrapping_sub(key_tick) as i32) > 0 {
                LastInput::Click
            } else {
                LastInput::Key(vk)
            }
        }
        (Some((vk, _)), None) => LastInput::Key(vk),
        (None, Some(_)) => LastInput::Click,
        (None, None) => LastInput::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latest_picks_the_newer_input_across_tick_wrap() {
        assert_eq!(latest(None, None), LastInput::None);
        assert_eq!(latest(Some((0x1B, 10)), None), LastInput::Key(0x1B));
        assert_eq!(latest(None, Some(10)), LastInput::Click);
        assert_eq!(latest(Some((0x0D, 10)), Some(20)), LastInput::Click);
        assert_eq!(latest(Some((0x0D, 20)), Some(10)), LastInput::Key(0x0D));
        assert_eq!(latest(Some((0x1B, u32::MAX)), Some(5)), LastInput::Click);
    }
}
