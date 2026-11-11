//! Preview, status and apply orchestration (05 §4.15).

use crate::{
    config::load::Config,
    fs::FileSystem,
    host::HostContext,
    layout::Layout,
    plan::{self, Choices, DesiredSet, Plan},
    preflight, resolve,
    state::State,
};

pub struct Analysis {
    pub layout: Layout,
    pub state: State,
    pub plan: Plan,
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
    Ok(Analysis {
        layout: config.layout,
        state,
        plan,
    })
}
