use super::distributed::WorldHeadAnnounce;

#[derive(Debug)]
pub struct HeadSyncResult<W> {
    pub head: WorldHeadAnnounce,
    pub world: W,
}

#[derive(Debug)]
pub struct HeadSyncReport<W> {
    pub drained: usize,
    pub applied: Option<HeadSyncResult<W>>,
}

#[derive(Debug)]
pub struct HeadFollowReport<W> {
    pub rounds: usize,
    pub drained: usize,
    pub applied: Option<HeadSyncResult<W>>,
}

pub fn compose_head_sync_report<W, E>(
    drained: usize,
    world: Option<W>,
    current_head: Option<WorldHeadAnnounce>,
    missing_head_error: impl FnOnce() -> E,
) -> Result<HeadSyncReport<W>, E> {
    let applied = world
        .map(|world| {
            current_head
                .ok_or_else(missing_head_error)
                .map(|head| HeadSyncResult { head, world })
        })
        .transpose()?;
    Ok(HeadSyncReport { drained, applied })
}

pub fn follow_head_sync<W, E>(
    max_rounds: usize,
    mut sync_once: impl FnMut() -> Result<HeadSyncReport<W>, E>,
) -> Result<HeadFollowReport<W>, E> {
    let mut rounds = 0usize;
    let mut drained = 0usize;
    let mut applied: Option<HeadSyncResult<W>> = None;
    for _ in 0..max_rounds {
        let report = sync_once()?;
        rounds += 1;
        drained += report.drained;
        if report.applied.is_some() {
            applied = report.applied;
        }
        if report.drained == 0 {
            break;
        }
    }
    Ok(HeadFollowReport {
        rounds,
        drained,
        applied,
    })
}
