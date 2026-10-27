use std::cell::RefCell;

use meta_log::LogStore as _;

use meta_pull_query::LiveQuery;
use meta_severity::Scales;
use meta_status::{Github, GithubFailure};

use super::{CLAIM, LINK, Meta, OPEN_DASHED, OWNER, PROGRAM, PULL_REQUEST};
use crate::request::Request;
use crate::request::output::Output;
use crate::request::status_view::StatusView;

const OFFLINE: &str = "offline";
const FLAGGED: &str = "unknown github: offline\n";
const HEADER: &str = "flags:\nunknown github: offline\nfeatures-and-maintenance\n  ";
const SHOWN_LINK: &str = " assigned | #12 unknown | held by owner | --dashed text\n";
const TALLY: &str = "severity, open concerns and issues (counts roll up):\n";
const STATUS_FLAGS: [&str; 3] = [PROGRAM, "status", "--flags"];

struct Asked(RefCell<String>);

impl Asked {
    fn answer(&self, meta: &Meta, view: StatusView) -> Output {
        let offline = |query: &LiveQuery| {
            self.0.replace(query.field());
            Github::Unreached(GithubFailure::new(OFFLINE))
        };
        view.answer(&meta.0, &Scales::default(), offline).unwrap()
    }
}

#[test]
fn status_asks_github_about_linked_pull_requests_and_flags_exit_apart_from_the_tree() {
    let meta = Meta::default();
    let task = meta.written(OWNER, &OPEN_DASHED);
    meta.run(OWNER, &[CLAIM, &task]).unwrap();
    meta.run(OWNER, &[LINK, &task, PULL_REQUEST]).unwrap();
    let asked = Asked(RefCell::default());
    let flags = asked.answer(&meta, StatusView::Flags);
    let tree = asked.answer(&meta, StatusView::Tree).to_string();
    assert!(matches!(&flags, Output::Flagged(_)) && flags.to_string() == FLAGGED && asked.0.borrow().contains("pr12:"));
    assert!(tree.starts_with(HEADER) && tree.ends_with(&format!("{task}{SHOWN_LINK}{TALLY}")));
    let parsed = Request::try_from(STATUS_FLAGS.map(String::from).to_vec()).unwrap();
    assert!(matches!(parsed, Request::Status(StatusView::Flags)));
}

#[test]
fn a_second_link_of_the_same_pull_request_is_refused_and_writes_nothing() {
    let meta = Meta::default();
    let task = meta.written(OWNER, &OPEN_DASHED);
    meta.run(OWNER, &[LINK, &task, PULL_REQUEST]).unwrap();
    let written = meta.0.read().unwrap().records().len();
    let refused = meta.run(OWNER, &[LINK, &task, PULL_REQUEST]).unwrap_err().to_string();
    assert!(refused.contains("#12 is already linked") && meta.0.read().unwrap().records().len() == written);
}
