use rosamta_core::{
    host::HostContext,
    plan::{Action, Disposition, Plan},
};
use serde_json::{Value, json};

use super::diff;

pub fn preview(plan: &Plan, host: &HostContext, show_diff: bool) -> (Value, String) {
    let mut lines = Vec::new();
    let mut patches = Vec::new();
    if plan.blocked {
        lines.push(format!(
            "strict mode: {} drift conflicts; apply would not write anything",
            plan.items.iter().filter(|item| item.drift).count()
        ));
    }
    let items = plan.items.iter().filter(|item| {
        show_diff || item.action != Action::Unchanged || item.record_changed || item.disposition == Disposition::Retain
    }).map(|item| {
        let mode = item.desired.as_ref().and_then(|desired| desired.mode).filter(|mode| item.before.as_ref().and_then(|entry| entry.mode) != Some(*mode))
            .map(|mode| json!({"from": item.before.as_ref().and_then(|entry| entry.mode).map(|mode| format!("{mode:04o}")), "to": format!("{mode:04o}")}));
        let reason = item.reason.map_or_else(String::new, |reason| format!(" ({reason})"));
        let mode_change = mode.as_ref().map_or_else(String::new, |mode| format!(" mode {} -> {}", mode["from"], mode["to"]));
        lines.push(format!("{:<10} {}  {}  {}{reason}{mode_change}", item.action.name(), item.target, item.output, item.disposition.name()));
        let mut json = json!({"output": item.output, "target": item.target.to_absolute(host), "origin": if item.cleanup { "cleanup" } else { "enabled" },
            "action": item.action.name(), "disposition": item.disposition.name(), "reason": item.reason, "entry_kind": item.before.as_ref().map(|entry| entry.kind.name()),
            "drift": item.drift, "mode": mode});
        if show_diff {
            let (patch, binary) = diff::unified(item);
            if let Some(patch) = &patch { patches.push(patch.trim_end_matches('\n').to_owned()); }
            if binary { patches.push(format!("Binary files differ: {}", item.target)); }
            json["diff"] = json!(patch);
            json["binary"] = json!(binary);
        }
        json
    }).collect::<Vec<_>>();
    if !patches.is_empty() {
        lines.push(String::new());
        lines.extend(patches);
    }
    (
        json!({"strict": plan.strict, "blocked": plan.blocked, "items": items}),
        lines.join("\n"),
    )
}

pub fn applied(
    plan: &Plan,
    host: &HostContext,
    report: &rosamta_core::exec::Report,
) -> (Value, String) {
    let mut lines = Vec::new();
    let items = plan.items.iter().zip(&report.items).map(|(item, result)| {
        let reason = item.reason.map_or_else(String::new, |reason| format!(" ({reason})"));
        let backup = result.backup.as_ref().map_or_else(String::new, |path| format!(" backup: {path}"));
        lines.push(format!("{:<10} {}  {}  {}{reason}{backup}", item.action.name(), item.target, item.output, result.status));
        json!({"output": item.output, "target": item.target.to_absolute(host), "action": item.action.name(), "status": result.status,
            "reason": item.reason, "entry_kind": item.before.as_ref().map(|entry| entry.kind.name()),
            "backup": result.backup.as_ref().map(|path| path.to_absolute(host))})
    }).collect::<Vec<_>>();
    let failure = report.failure.as_ref().map(|failure| {
        json!({
            "code": "exec.failed", "item": failure.item.map(|index| &plan.items[index].output),
            "path": failure.path.to_absolute(host), "message": failure.message, "os_error": null
        })
    });
    let recovery = json!({"state": if report.restored { "restored" } else { "untouched" },
        "failures": report.recovery.iter().map(|failure| json!({"path": failure.path.to_absolute(host), "reason": failure.reason,
            "backup": report.items[failure.index].backup.as_ref().map(|path| path.to_absolute(host))})).collect::<Vec<_>>()});
    (
        json!({"batch": report.batch, "outcome": report.outcome(), "items": items, "failure": failure, "recovery": recovery}),
        lines.join("\n"),
    )
}
