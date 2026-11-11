//! Preview, status and apply orchestration (05 §4.15).

use crate::{
    config::load::Config,
    fs::FileSystem,
    host::HostContext,
    layout::Layout,
    plan::{self, DesiredSet, Plan},
    preflight, resolve,
    state::State,
};

pub use crate::plan::Choices;

pub struct Analysis {
    pub layout: Layout,
    pub state: State,
    pub plan: Plan,
}

pub type Deployment = Result<(Analysis, crate::exec::Report), Vec<String>>;

pub fn deploy(
    fs: &dyn FileSystem,
    host: &HostContext,
    choices: &Choices,
    batch: &str,
) -> Deployment {
    let (layout, _) = crate::config::load::load_layout(fs, host).map_err(|error| vec![error])?;
    crate::config::load::load_manifest(fs, &layout).map_err(|error| vec![error])?;
    let lock = layout.state.join("lock").map_err(|error| vec![error])?;
    let _lock = fs
        .lock(&lock)
        .map_err(|error| vec![format!("{lock}: {error}")])?;
    execute_locked(fs, host, choices, batch)
}

pub(crate) fn execute_locked(
    fs: &dyn FileSystem,
    host: &HostContext,
    choices: &Choices,
    batch: &str,
) -> Deployment {
    let analysis = analyze(fs, host, choices)?;
    let report = crate::exec::execute(
        fs,
        host.platform,
        &analysis.layout,
        &analysis.state,
        &analysis.plan,
        batch,
    )
    .map_err(|error| vec![error])?;
    Ok((analysis, report))
}

pub fn analyze(
    fs: &dyn FileSystem,
    host: &HostContext,
    choices: &Choices,
) -> Result<Analysis, Vec<String>> {
    let config = Config::load(fs, host)?;
    let state = State::load(fs, &config.layout, host.platform).map_err(|error| vec![error])?;
    let resolved = resolve::resolve(&config, host)?;
    let desired = preflight::desired(fs, host, &config, &resolved)?;
    let mut plan = plan::build(
        fs,
        &config.layout,
        host,
        &state,
        DesiredSet(desired),
        choices,
    )?;
    for output in config.enabled(host).1 {
        plan.warnings.push(format!(
            "profile enables unknown or removed output {output}"
        ));
    }
    if let Ok(path) = config.layout.state.join("txn")
        && let Ok(entries) = fs.children(&path)
        && !entries.is_empty()
    {
        plan.warnings.push(format!(
            "{path}: previous transaction snapshots remain; inspect them before removal"
        ));
    }
    Ok(Analysis {
        layout: config.layout,
        state,
        plan,
    })
}
