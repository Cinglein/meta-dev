use std::cell::RefCell;
use std::time::Duration;

use meta_golden::Golden;
use meta_log::LogStore as _;
use meta_pull_query::LiveQuery;
use meta_severity::Scales;
use meta_status::{Github, GithubFailure};

use super::{CLAIM, LINK, Meta, OPEN_DASHED, OWNER, PROGRAM, PULL_REQUEST};
use crate::request::Request;
use crate::request::output::Output;
use crate::request::status_view::StatusView;

const WITHIN: Duration = Duration::from_secs(300);
const OFFLINE: &str = "offline";
const TASK: &str = "$TASK";
const ASKED: &str = "pr12:";
const STATUS_FLAGS: [&str; 3] = [PROGRAM, "status", "--flags"];

struct Asked(RefCell<String>);

impl Asked {
    fn answer(&self, meta: &Meta, view: StatusView) -> Output {
        let offline = |query: &LiveQuery| {
            self.0.replace(query.field());
            Github::Unreached(GithubFailure::new(OFFLINE))
        };
        view.answer(&meta.0, &Scales::default(), WITHIN, offline).unwrap()
    }
}

#[test]
fn status_asks_github_about_linked_pull_requests_and_a_second_link_of_one_is_refused() {
    let meta = Meta::default();
    let task = meta.written(OWNER, &OPEN_DASHED);
    meta.run(OWNER, &[CLAIM, &task]).unwrap();
    meta.run(OWNER, &[LINK, &task, PULL_REQUEST]).unwrap();
    let asked = Asked(RefCell::default());
    let [flags, tree] = [StatusView::Flags, StatusView::Tree].map(|view| asked.answer(&meta, view));
    let parsed = Request::try_from(STATUS_FLAGS.map(String::from).to_vec());
    let written = meta.0.read().unwrap().records().len();
    let refused = meta.run(OWNER, &[LINK, &task, PULL_REQUEST]).map_err(|failure| failure.to_string());
    let shown = format!("{flags:?}\n{tree}{parsed:?}\n{refused:?}\n").replace(&task, TASK);
    Golden::for_test(module_path!(), "status").checked(&shown).unwrap();
    assert!(asked.0.borrow().contains(ASKED) && meta.0.read().unwrap().records().len() == written);
}
