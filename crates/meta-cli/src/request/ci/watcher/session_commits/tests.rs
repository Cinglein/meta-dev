use meta_golden::{Goldens, Recording};
use meta_log::RemoteHeads;
use meta_run::GitArgv;

use super::SessionCommits;
use crate::request::ci::watcher::reach::Reach;
use crate::request::ci::{Commit, ReflogEntry, StartMoment};

const SCENARIO: &str = "ci-session";
const CREATED: [&str; 6] = [
    "f2e83b47f04f1aadaaf72b9d15074e933a513dc6",
    "9271c82b76058cb9022078bf58c641cfc6a5b0d7",
    "0535038a5657a4872055f5ca2aa4ae04f811e419",
    "61492af0ca298e0ad04f94f231edcc3b7717b88b",
    "35f3121103217e323172c65bc2941330453b4be0",
    "c3d2fcde5f8b6fb687879d74ce45f40c0faafc2d",
];
const BEFORE: &str = "1699999999999999999";
const SAME_SECOND: &str = "1700000000000000000";
const AFTER: &str = "1700000001000000000";
const PUSHED_BY_HEADS: [&str; 3] = [CREATED[2], CREATED[4], CREATED[5]];
const INDEPENDENT: [&str; 1] = [CREATED[2]];
const TIPS: [&str; 2] = [CREATED[2], CREATED[5]];

struct Recorded;

impl Recorded {
    fn outs(call: GitArgv) -> Vec<String> {
        let recordings = Recording::all_in(&Goldens::workspace().dir().join(SCENARIO)).unwrap();
        recordings.iter().filter(|recording| recording.argv().calls(call)).map(|found| found.out().to_owned()).collect()
    }

    pub fn commits(listed: &[&str]) -> Vec<Commit> {
        listed.iter().map(|sha| sha.parse().unwrap()).collect()
    }

    pub fn listed<Line: std::str::FromStr<Err: std::fmt::Debug>>(call: GitArgv) -> Vec<Vec<Line>> {
        Self::outs(call).iter().map(|out| out.lines().map(|line| line.parse().unwrap()).collect()).collect()
    }

    fn walk() -> Vec<ReflogEntry> {
        Self::listed(GitArgv::ReflogWalk).into_iter().next().unwrap()
    }

    fn created(entries: &[ReflogEntry], start: &str, born: &[&str]) -> Vec<Commit> {
        let born = Self::commits(born);
        ReflogEntry::created(entries, &start.parse::<StartMoment>().unwrap(), |commit| born.contains(commit))
    }
}

#[test]
fn every_commit_made_after_the_start_counts_once_whatever_its_subject_or_dates_and_the_start_second_asks_its_object() {
    let walk = Recorded::walk();
    let [before, after] = [BEFORE, AFTER].map(|start| Recorded::created(&walk, start, &[]));
    let same_second = Recorded::created(&walk, SAME_SECOND, &CREATED[1..2]);
    let merged_elsewhere: Vec<ReflogEntry> = walk.iter().take(1).chain(walk.iter().skip(2)).cloned().collect();
    let merged_elsewhere = Recorded::created(&merged_elsewhere, BEFORE, &[]);
    let expected = [&CREATED[..], &CREATED[..0], &CREATED[1..2], &CREATED[1..]].map(Recorded::commits);
    assert_eq!([before, after, same_second, merged_elsewhere], expected);
    assert_eq!(SAME_SECOND.parse::<StartMoment>().map(|start| start.to_string()), Ok(SAME_SECOND.to_owned()));
}

#[test]
fn commits_the_remote_holds_on_any_branch_count_as_pushed_and_only_independent_tips_are_watched() {
    let created = Recorded::commits(&CREATED);
    let heads: RemoteHeads = Recorded::outs(GitArgv::RemoteHeads).concat().parse().unwrap();
    let checkout = Recorded::walk().into_iter().find_map(|entry| entry.checkout().cloned()).unwrap();
    let judged = Recorded::listed(GitArgv::Unpushed).into_iter();
    let judged: Vec<SessionCommits> =
        judged.map(|unpushed| SessionCommits::new(&created, Some(unpushed), checkout.clone(), heads.clone())).collect();
    let pushed: Vec<Vec<Commit>> = judged.iter().map(|seen| seen.pushed().to_vec()).collect();
    let tips = Recorded::listed(GitArgv::IndependentTips);
    let tipped = judged.into_iter().rev().zip(tips).map(|(seen, tips)| seen.tipped(Some(tips)));
    assert_eq!(pushed, [Recorded::commits(&CREATED[5..]), Recorded::commits(&PUSHED_BY_HEADS)]);
    let asked = [Reach::Remote, Reach::Tracking].map(|reach| reach.asks_remote(&created));
    assert_eq!((heads.tips::<Commit>(), asked), (Recorded::commits(&TIPS), [Some(()), None]));
    let tipped: Vec<_> = tipped
        .map(|seen| {
            (seen.unpushed_head().cloned(), seen.pushed().to_vec(), seen.heads().clone(), seen.checkout().clone())
        })
        .collect();
    assert_eq!(tipped, [(created.first().cloned(), Recorded::commits(&INDEPENDENT), heads, checkout)]);
    let known = [Some(created.clone()), None].map(|created| SessionCommits::known(created).ok());
    assert_eq!(known, [Some(created), None]);
}
