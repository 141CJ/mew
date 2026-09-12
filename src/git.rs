use std::path::{Path, PathBuf};

use anyhow::Result;
use git2::{BranchType, Repository, Status, StatusOptions};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MascotState {
    Conflict,
    Dirty,
    StagedOnly,
    AheadOnly,
    CleanWithTime,
    CleanNoTime,
    NoRepo,
}

#[derive(Debug, Clone, Default)]
pub struct GitAheadBehind {
    pub ahead: usize,
    pub behind: usize,
    pub has_upstream: bool,
    pub upstream_name: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct GitState {
    pub is_repo: bool,
    pub repo_root: Option<PathBuf>,
    pub branch_name: String,
    pub commit_hash: String,
    pub is_detached: bool,
    pub ahead_behind: GitAheadBehind,
    pub modified_count: usize,
    pub staged_count: usize,
    pub untracked_count: usize,
    pub conflict_count: usize,
}

pub fn detect_git_state(dir: &Path) -> Result<GitState> {
    let repo = match Repository::discover(dir) {
        Ok(r) => r,
        Err(_) => return Ok(GitState::default()),
    };

    let repo_root = repo.workdir().map(|p| p.to_path_buf());

    let (branch_name, commit_hash, is_detached) = match repo.head() {
        Ok(head) => {
            let target_oid = head.target();
            let short_sha = match target_oid {
                Some(oid) => {
                    let sha = oid.to_string();
                    if sha.len() >= 7 {
                        sha[..7].to_string()
                    } else {
                        sha
                    }
                }
                None => "—".to_string(),
            };

            if head.is_branch() {
                let name = head.shorthand().unwrap_or("unknown").to_lowercase();
                (name, short_sha, false)
            } else {
                (format!("detached@{}", short_sha), short_sha, true)
            }
        }
        Err(_) => ("main".to_string(), "—".to_string(), false),
    };

    // upstream presence and ahead/behind collapse into one lookup: an
    // existing upstream keeps its name even when oid resolution or the
    // graph walk fails (0/0), exactly as before, with less nesting.
    let ahead_behind = if is_detached {
        GitAheadBehind::default()
    } else {
        match repo
            .find_branch(&branch_name, BranchType::Local)
            .ok()
            .and_then(|local| local.upstream().ok().map(|upstream| (local, upstream)))
        {
            Some((local, upstream)) => {
                let upstream_name = upstream.name().ok().flatten().map(|s| s.to_lowercase());
                let (ahead, behind) = match (local.get().target(), upstream.get().target()) {
                    (Some(l), Some(u)) => repo.graph_ahead_behind(l, u).unwrap_or((0, 0)),
                    _ => (0, 0),
                };
                GitAheadBehind {
                    ahead,
                    behind,
                    has_upstream: true,
                    upstream_name,
                }
            }
            None => GitAheadBehind::default(),
        }
    };

    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(false)
        .include_ignored(false);

    let statuses = repo.statuses(Some(&mut opts))?;

    let mut modified_count = 0;
    let mut staged_count = 0;
    let mut untracked_count = 0;
    let mut conflict_count = 0;

    for entry in statuses.iter() {
        let s = entry.status();

        if s.contains(Status::CONFLICTED) {
            conflict_count += 1;
            continue;
        }

        if s.intersects(
            Status::INDEX_NEW
                | Status::INDEX_MODIFIED
                | Status::INDEX_DELETED
                | Status::INDEX_RENAMED
                | Status::INDEX_TYPECHANGE,
        ) {
            staged_count += 1;
        }

        if s.intersects(
            Status::WT_MODIFIED | Status::WT_DELETED | Status::WT_RENAMED | Status::WT_TYPECHANGE,
        ) {
            modified_count += 1;
        }

        if s.contains(Status::WT_NEW) {
            untracked_count += 1;
        }
    }

    Ok(GitState {
        is_repo: true,
        repo_root,
        branch_name,
        commit_hash,
        is_detached,
        ahead_behind,
        modified_count,
        staged_count,
        untracked_count,
        conflict_count,
    })
}

pub fn compute_mascot_state(git: &GitState, has_time_widget: bool) -> MascotState {
    if !git.is_repo {
        return MascotState::NoRepo;
    }

    if git.conflict_count > 0 {
        MascotState::Conflict
    } else if git.modified_count > 0 || git.untracked_count > 0 {
        MascotState::Dirty
    } else if git.staged_count > 0 {
        MascotState::StagedOnly
    } else if git.ahead_behind.ahead > 0 {
        MascotState::AheadOnly
    } else if has_time_widget {
        MascotState::CleanWithTime
    } else {
        MascotState::CleanNoTime
    }
}

pub struct MascotVisual {
    pub middle_line: &'static str,
    pub mood_label: &'static str,
    pub color_role: &'static str,
}

pub fn mascot_visual_for_state(state: MascotState) -> MascotVisual {
    match state {
        MascotState::Conflict => MascotVisual {
            middle_line: "( >_< )",
            mood_label: "conflict",
            color_role: "status_error",
        },
        MascotState::Dirty => MascotVisual {
            middle_line: "( o.o )",
            mood_label: "watching",
            color_role: "status_dirty",
        },
        MascotState::StagedOnly => MascotVisual {
            middle_line: "( ~_~ )",
            mood_label: "ready",
            color_role: "status_clean",
        },
        MascotState::AheadOnly => MascotVisual {
            middle_line: "( ^.^)~",
            mood_label: "ship",
            color_role: "gauge_fill",
        },
        MascotState::CleanWithTime => MascotVisual {
            middle_line: "( -.~ )",
            mood_label: "ticking",
            color_role: "task_active",
        },
        MascotState::CleanNoTime => MascotVisual {
            middle_line: "( ^.^ )",
            mood_label: "all clear",
            color_role: "status_clean",
        },
        MascotState::NoRepo => MascotVisual {
            middle_line: "( =.= )",
            mood_label: "no repo",
            color_role: "muted",
        },
    }
}
