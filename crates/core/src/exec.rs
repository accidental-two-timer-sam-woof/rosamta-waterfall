//! Plan execution and takeover backups (05 §4.14).

use crate::{
    fs::{Entry, FileSystem, Kind, Replacement},
    host::Platform,
    layout::Layout,
    path::HomePath,
    plan::{self, Disposition, Item, Plan},
    recovery::{Journal, RecoveryFailure, unique_directory},
    state::State,
};

pub struct ItemReport {
    pub status: &'static str,
    pub backup: Option<HomePath>,
}

pub struct Failure {
    pub item: Option<usize>,
    pub path: HomePath,
    pub message: String,
}

pub struct Report {
    pub batch: String,
    pub items: Vec<ItemReport>,
    pub failure: Option<Failure>,
    pub recovery: Vec<RecoveryFailure>,
    pub restored: bool,
    pub warnings: Vec<String>,
}

impl Report {
    pub fn exit_code(&self) -> u8 {
        let retained = self
            .items
            .iter()
            .filter(|item| item.status == "retained")
            .count();
        if self.failure.is_some() || (retained > 0 && retained == self.items.len()) {
            1
        } else if retained > 0 {
            2
        } else {
            0
        }
    }

    pub fn outcome(&self) -> &'static str {
        match self.exit_code() {
            0 => "completed",
            2 => "partial",
            _ => "failed",
        }
    }
}

pub fn execute(
    fs: &dyn FileSystem,
    platform: Platform,
    layout: &Layout,
    state: &State,
    plan: &Plan,
    batch: &str,
) -> Result<Report, String> {
    if plan.blocked {
        let targets = plan
            .items
            .iter()
            .filter(|item| item.drift)
            .map(|item| format!("{} {}", item.output, item.target))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!("strict mode blocked by drift: {targets}"));
    }
    let mut report = Report {
        batch: batch.to_owned(),
        items: plan
            .items
            .iter()
            .map(|item| ItemReport {
                status: if item.disposition == Disposition::Retain {
                    "retained"
                } else {
                    "not-run"
                },
                backup: None,
            })
            .collect(),
        failure: None,
        recovery: Vec::new(),
        restored: false,
        warnings: Vec::new(),
    };
    if !plan
        .items
        .iter()
        .any(|item| item.writes_target() || item.record_changed)
    {
        for item in &mut report.items {
            if item.status != "retained" {
                item.status = "done";
            }
        }
        return Ok(report);
    }
    let (batch, dir) = unique_directory(fs, &layout.state.join("txn")?, batch)?;
    report.batch = batch;
    let mut journal = Journal::new(fs, dir);
    let mut records = state.records.clone();
    let mut check_after = Vec::new();

    for (index, item) in plan.items.iter().enumerate() {
        if item.disposition == Disposition::Retain {
            continue;
        }
        match apply_item(
            fs,
            layout,
            &report.batch,
            index,
            item,
            &mut journal,
            &mut report.items[index],
        ) {
            Ok(current) => {
                match plan::update_record(&mut records, item, current.as_ref(), platform) {
                    Ok(()) => {
                        if item.writes_target() || item.record_changed {
                            check_after.push((index, current));
                        }
                        report.items[index].status = "done";
                    }
                    Err(error) => {
                        fail(&mut report, index, item, error);
                        break;
                    }
                }
            }
            Err(error) => {
                fail(&mut report, index, item, error);
                break;
            }
        }
    }

    if report.failure.is_none() {
        for (index, expected) in check_after {
            let item = &plan.items[index];
            // A declared mode of 000 can prevent re-reading the file we just wrote.
            if expected.as_ref().is_some_and(|entry| entry.mode == Some(0)) {
                continue;
            }
            if !matches!(fs.inspect(&item.target), Ok(actual) if actual == expected) {
                fail(
                    &mut report,
                    index,
                    item,
                    "target changed before committing state".into(),
                );
                break;
            }
        }
    }
    if report.failure.is_none()
        && records != state.records
        && let Err(error) = write_state(fs, state, &records)
    {
        report.failure = Some(Failure {
            item: None,
            path: state.path.clone(),
            message: error,
        });
    }
    if report.failure.is_some() {
        let (reverted, failures) = journal.rollback();
        for index in reverted {
            report.items[index].status = "reverted";
        }
        for failure in &failures {
            report.items[failure.index].status = "not-reverted";
        }
        report.restored = failures.is_empty();
        report.recovery = failures;
    }
    if report.recovery.is_empty()
        && let Err(error) = journal.finish()
    {
        report.warnings.push(format!(
            "transaction snapshots could not be cleaned: {error}"
        ));
    }
    Ok(report)
}

fn fail(report: &mut Report, index: usize, item: &Item, error: String) {
    report.items[index].status = "failed";
    report.failure = Some(Failure {
        item: Some(index),
        path: item.target.clone(),
        message: error,
    });
}

fn apply_item(
    fs: &dyn FileSystem,
    layout: &Layout,
    batch: &str,
    index: usize,
    item: &Item,
    journal: &mut Journal<'_>,
    report: &mut ItemReport,
) -> Result<Option<Entry>, String> {
    let checked = fs
        .inspect(&item.target)
        .map_err(|error| error.to_string())?;
    if checked != item.before {
        return Err("target changed after planning".into());
    }
    if !item.writes_target() {
        return Ok(checked);
    }
    let mut link_backup = None;
    if let Some(before) = &checked {
        if before.kind == Kind::Regular {
            journal.save(index, before)?;
        }
        if item.disposition == Disposition::Takeover {
            let path = layout
                .state
                .join("backups")?
                .join(batch)?
                .join(item.target.as_str())?;
            if before.kind == Kind::Symlink {
                fs.backup_link(&item.target, before, &path)
                    .map_err(|error| format!("{path}: {error}"))?;
                link_backup = Some(path.clone());
            } else {
                fs.change(
                    &path,
                    None,
                    Some(Replacement {
                        bytes: &before.bytes,
                        mode: before.mode,
                    }),
                )
                .map_err(|error| format!("{path}: {error}"))?;
            }
            report.backup = Some(path);
        }
    }
    let replacement = item.desired.as_ref().map(|desired| Replacement {
        bytes: &desired.bytes,
        mode: desired.mode,
    });
    let after = fs
        .change(&item.target, checked.as_ref(), replacement)
        .map_err(|error| error.to_string())?;
    journal.record(
        index,
        item.target.clone(),
        checked,
        after.clone(),
        link_backup,
    );
    Ok(after)
}

pub fn write_state(
    fs: &dyn FileSystem,
    state: &State,
    records: &std::collections::BTreeMap<String, crate::state::Record>,
) -> Result<(), String> {
    let bytes = State::encode(records)?;
    fs.change(
        &state.path,
        state.before.as_ref(),
        Some(Replacement {
            bytes: &bytes,
            mode: Some(0o600),
        }),
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}
