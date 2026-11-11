//! Unified diff output (05 §9.4).

use rosamta_core::plan::{Action, Item};
use similar::TextDiff;

pub fn unified(item: &Item) -> (Option<String>, bool) {
    if item.action == Action::Unchanged {
        return (None, false);
    }
    let before = item
        .before
        .as_ref()
        .map_or(&[][..], |entry| entry.bytes.as_slice());
    let after = item
        .desired
        .as_ref()
        .map_or(&[][..], |desired| desired.bytes.as_slice());
    if before == after {
        return (None, false);
    }
    let (Ok(before), Ok(after)) = (std::str::from_utf8(before), std::str::from_utf8(after)) else {
        return (None, true);
    };
    if before.contains('\0') || after.contains('\0') {
        return (None, true);
    }
    let old = if item.before.is_none() {
        "/dev/null".into()
    } else {
        format!("a/{}", item.target.as_str())
    };
    let new = if item.action == Action::Delete {
        "/dev/null".into()
    } else {
        format!("b/{}", item.target.as_str())
    };
    let diff = TextDiff::from_lines(before, after)
        .unified_diff()
        .header(&old, &new)
        .to_string();
    (Some(diff), false)
}
