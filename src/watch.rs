//! Infinite polling watch coroutine: baseline via `users.getProfile`,
//! then poll `users.history.list` on a timer (yielding `WantsSleep`) and
//! emit one raw `GmailHistoryDiff` per tick.
//!
//! Gmail sync guide: <https://developers.google.com/gmail/api/guides/sync>

use core::{convert::Infallible, fmt, mem, time::Duration};

use alloc::{string::String, vec::Vec};

use log::trace;
use secrecy::SecretString;
use thiserror::Error;

use crate::{
    coroutine::{GmailCoroutine, GmailCoroutineState, GmailYield},
    history::{GmailHistoryLabel, list::GmailHistoryList},
    messages::{GmailMessage, GmailMessageFormat, GmailMessageId, get::GmailMessageGet},
    profile::GmailProfileGet,
    send::GmailSendError,
};

const POLL_SECONDS: u64 = 30;

#[derive(Debug, Error)]
pub enum GmailWatchError {
    #[error(transparent)]
    Send(#[from] GmailSendError),
}

#[derive(Clone, Debug, Default)]
pub struct GmailHistoryDiff {
    pub history_id: String,
    pub added: Vec<GmailMessage>,
    pub removed: Vec<GmailMessageId>,
    pub labels_added: Vec<GmailHistoryLabel>,
    pub labels_removed: Vec<GmailHistoryLabel>,
}

#[derive(Debug)]
pub enum GmailWatchYield {
    WantsRead,
    WantsWrite(Vec<u8>),
    WantsSleep(Duration),
    Diff(GmailHistoryDiff),
}

pub struct GmailWatch {
    state: State,
    http_auth: SecretString,
    user_id: String,
    mailbox: String,
    history_id: Option<String>,
}

impl GmailWatch {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        mailbox: &str,
    ) -> Result<Self, GmailWatchError> {
        trace!("prepare Gmail watch");
        let profile = GmailProfileGet::new(http_auth, user_id)?;
        Ok(Self {
            state: State::Baseline(profile),
            http_auth: http_auth.clone(),
            user_id: user_id.into(),
            mailbox: mailbox.into(),
            history_id: None,
        })
    }

    fn history_list(&self, page_token: Option<&str>) -> Result<GmailHistoryList, GmailSendError> {
        let since = self.history_id.as_deref().unwrap_or_default();
        GmailHistoryList::new(
            &self.http_auth,
            &self.user_id,
            since,
            Some(&self.mailbox),
            &[],
            None,
            page_token,
        )
    }

    fn message_get(&self, id: &str) -> Result<GmailMessageGet, GmailSendError> {
        GmailMessageGet::new(
            &self.http_auth,
            &self.user_id,
            id,
            GmailMessageFormat::Metadata,
            &[],
        )
    }

    fn finalize(&mut self, cycle: Cycle) -> GmailHistoryDiff {
        let history_id = cycle
            .new_history_id
            .or_else(|| self.history_id.clone())
            .unwrap_or_default();
        self.history_id = Some(history_id.clone());
        self.state = State::Sleeping;
        GmailHistoryDiff {
            history_id,
            added: cycle.added,
            removed: cycle.removed,
            labels_added: cycle.labels_added,
            labels_removed: cycle.labels_removed,
        }
    }
}

impl GmailCoroutine for GmailWatch {
    type Yield = GmailWatchYield;
    type Return = Result<Infallible, GmailWatchError>;

    fn resume(&mut self, bytes: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        trace!("watch: {}", self.state);
        let mut bytes = bytes;
        loop {
            match mem::replace(&mut self.state, State::Done) {
                State::Baseline(mut profile) => match profile.resume(bytes.take()) {
                    GmailCoroutineState::Yielded(GmailYield::WantsRead) => {
                        self.state = State::Baseline(profile);
                        return GmailCoroutineState::Yielded(GmailWatchYield::WantsRead);
                    }
                    GmailCoroutineState::Yielded(GmailYield::WantsWrite(out)) => {
                        self.state = State::Baseline(profile);
                        return GmailCoroutineState::Yielded(GmailWatchYield::WantsWrite(out));
                    }
                    GmailCoroutineState::Complete(Err(err)) => {
                        return GmailCoroutineState::Complete(Err(err.into()));
                    }
                    GmailCoroutineState::Complete(Ok(out)) => {
                        self.history_id = out.response.history_id;
                        self.state = State::Sleeping;
                    }
                },
                State::Sleeping => {
                    let list = match self.history_list(None) {
                        Ok(list) => list,
                        Err(err) => return GmailCoroutineState::Complete(Err(err.into())),
                    };
                    self.state = State::Listing {
                        list,
                        cycle: Cycle::default(),
                    };
                    return GmailCoroutineState::Yielded(GmailWatchYield::WantsSleep(
                        Duration::from_secs(POLL_SECONDS),
                    ));
                }
                State::Listing {
                    mut list,
                    mut cycle,
                } => match list.resume(bytes.take()) {
                    GmailCoroutineState::Yielded(GmailYield::WantsRead) => {
                        self.state = State::Listing { list, cycle };
                        return GmailCoroutineState::Yielded(GmailWatchYield::WantsRead);
                    }
                    GmailCoroutineState::Yielded(GmailYield::WantsWrite(out)) => {
                        self.state = State::Listing { list, cycle };
                        return GmailCoroutineState::Yielded(GmailWatchYield::WantsWrite(out));
                    }
                    GmailCoroutineState::Complete(Err(err)) => {
                        if err.status() == Some(404) {
                            trace!("gmail history cursor expired; re-baselining");
                            let profile = match GmailProfileGet::new(&self.http_auth, &self.user_id)
                            {
                                Ok(profile) => profile,
                                Err(err) => {
                                    return GmailCoroutineState::Complete(Err(err.into()));
                                }
                            };
                            self.history_id = None;
                            self.state = State::Baseline(profile);
                            continue;
                        }
                        return GmailCoroutineState::Complete(Err(err.into()));
                    }
                    GmailCoroutineState::Complete(Ok(out)) => {
                        let response = out.response;

                        for record in &response.history {
                            for message in &record.messages_added {
                                cycle.added_ids.push(message.message.id.clone());
                            }
                            for message in &record.messages_deleted {
                                cycle.removed.push(GmailMessageId {
                                    id: message.message.id.clone(),
                                    thread_id: message.message.thread_id.clone(),
                                });
                            }
                            for label in &record.labels_added {
                                cycle.labels_added.push(label.clone());
                            }
                            for label in &record.labels_removed {
                                cycle.labels_removed.push(label.clone());
                            }
                        }

                        if let Some(token) = response.next_page_token {
                            let list = match self.history_list(Some(&token)) {
                                Ok(list) => list,
                                Err(err) => {
                                    return GmailCoroutineState::Complete(Err(err.into()));
                                }
                            };
                            self.state = State::Listing { list, cycle };
                            continue;
                        }

                        cycle.new_history_id = response.history_id;

                        if cycle.added_ids.is_empty() {
                            let diff = self.finalize(cycle);
                            return GmailCoroutineState::Yielded(GmailWatchYield::Diff(diff));
                        }

                        let ids = mem::take(&mut cycle.added_ids);
                        let current = match self.message_get(&ids[0]) {
                            Ok(get) => get,
                            Err(err) => return GmailCoroutineState::Complete(Err(err.into())),
                        };
                        self.state = State::Fetching {
                            ids,
                            index: 0,
                            current,
                            cycle,
                        };
                    }
                },
                State::Fetching {
                    ids,
                    index,
                    mut current,
                    mut cycle,
                } => match current.resume(bytes.take()) {
                    GmailCoroutineState::Yielded(GmailYield::WantsRead) => {
                        self.state = State::Fetching {
                            ids,
                            index,
                            current,
                            cycle,
                        };
                        return GmailCoroutineState::Yielded(GmailWatchYield::WantsRead);
                    }
                    GmailCoroutineState::Yielded(GmailYield::WantsWrite(out)) => {
                        self.state = State::Fetching {
                            ids,
                            index,
                            current,
                            cycle,
                        };
                        return GmailCoroutineState::Yielded(GmailWatchYield::WantsWrite(out));
                    }
                    GmailCoroutineState::Complete(result) => {
                        match result {
                            Ok(out) => cycle.added.push(out.response),
                            // NOTE: a just-added message may already be gone
                            // by the time we fetch it; skip it rather than
                            // tearing the watch down.
                            Err(err) => trace!("gmail watch: skipping message get: {err}"),
                        }

                        let index = index + 1;
                        if index < ids.len() {
                            let current = match self.message_get(&ids[index]) {
                                Ok(get) => get,
                                Err(err) => {
                                    return GmailCoroutineState::Complete(Err(err.into()));
                                }
                            };
                            self.state = State::Fetching {
                                ids,
                                index,
                                current,
                                cycle,
                            };
                        } else {
                            let diff = self.finalize(cycle);
                            return GmailCoroutineState::Yielded(GmailWatchYield::Diff(diff));
                        }
                    }
                },
                // SAFETY: every arm reassigns `state` before yielding or
                // continuing, so the watch never rests in `Done`.
                State::Done => unreachable!("gmail watch resumed in terminal state"),
            }
        }
    }
}

#[derive(Default)]
struct Cycle {
    added_ids: Vec<String>,
    added: Vec<GmailMessage>,
    removed: Vec<GmailMessageId>,
    labels_added: Vec<GmailHistoryLabel>,
    labels_removed: Vec<GmailHistoryLabel>,
    new_history_id: Option<String>,
}

enum State {
    Baseline(GmailProfileGet),
    Sleeping,
    Listing {
        list: GmailHistoryList,
        cycle: Cycle,
    },
    Fetching {
        ids: Vec<String>,
        index: usize,
        current: GmailMessageGet,
        cycle: Cycle,
    },
    Done,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Baseline(_) => f.write_str("baseline"),
            Self::Sleeping => f.write_str("sleeping"),
            Self::Listing { .. } => f.write_str("listing"),
            Self::Fetching { .. } => f.write_str("fetching"),
            Self::Done => f.write_str("done"),
        }
    }
}
