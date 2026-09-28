//! Infinite polling watch coroutine built on the history API.
//!
//! Baselines the history cursor via `users.getProfile`, then polls
//! `users.history.list` on a timer (yielding `WantsSleep`) and emits
//! one raw `GmailHistoryDiff` per tick.
//!
//! Gmail sync guide: <https://developers.google.com/gmail/api/guides/sync>

use core::{convert::Infallible, mem, time::Duration};

use alloc::{string::String, vec::Vec};

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use thiserror::Error;

use crate::{
    coroutine::*,
    v1::{
        rest::{
            history::{
                GmailHistoryLabel,
                list::{GmailHistoryList, GmailHistoryListParams},
            },
            messages::{GmailMessage, GmailMessageFormat, GmailMessageId, get::GmailMessageGet},
            users::get_profile::GmailProfileGet,
        },
        send::GmailSendError,
    },
};

const POLL_SECONDS: u64 = 30;

/// Errors that can occur during the watch.
#[derive(Debug, Error)]
pub enum GmailHistoryPollError {
    /// One of the underlying Gmail exchanges failed.
    #[error(transparent)]
    Send(#[from] GmailSendError),
}

/// One tick's worth of mailbox changes, Gmail-native.
///
/// Consumers translate it into their own change representation; io-gmail
/// does not interpret it further.
#[derive(Clone, Debug, Default)]
pub struct GmailHistoryDiff {
    /// The history cursor after this diff, to persist for resuming.
    pub history_id: String,
    /// Messages added to the mailbox since the last tick.
    pub added: Vec<GmailMessage>,
    /// Messages removed from the mailbox since the last tick.
    pub removed: Vec<GmailMessageId>,
    /// Label additions on individual messages.
    pub labels_added: Vec<GmailHistoryLabel>,
    /// Label removals on individual messages.
    pub labels_removed: Vec<GmailHistoryLabel>,
}

/// I/O request or event yielded by the watch.
#[derive(Debug)]
pub enum GmailHistoryPollYield {
    /// The watch wants bytes read from the stream.
    WantsRead,
    /// The watch wants the given bytes written to the stream.
    WantsWrite(Vec<u8>),
    /// Asks the caller to sleep until the next poll.
    WantsSleep(Duration),
    /// One tick's worth of changes; the watch then goes back to sleep.
    Diff(GmailHistoryDiff),
}

/// I/O-free coroutine watching a mailbox by polling `users.history.list`.
///
/// Never completes successfully (its return type is `Infallible`): it
/// yields one [`GmailHistoryDiff`] per tick and re-baselines itself when
/// the server reports an expired history cursor.
pub struct GmailHistoryPoll {
    state: State,
    auth: HttpAuthBearer,
    user_id: String,
    mailbox: String,
    history_id: Option<String>,
}

impl GmailHistoryPoll {
    /// Builds the watch over the given mailbox label, baselining the
    /// history cursor first.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        mailbox: &str,
    ) -> Result<Self, GmailHistoryPollError> {
        debug!("prepare gmail poll history");
        trace!("user_id: {user_id:?}");
        trace!("mailbox: {mailbox:?}");

        let profile = GmailProfileGet::new(auth, user_id)?;
        Ok(Self {
            state: State::Baseline(profile),
            auth: auth.clone(),
            user_id: user_id.into(),
            mailbox: mailbox.into(),
            history_id: None,
        })
    }

    fn list_history(&self, page_token: Option<&str>) -> Result<GmailHistoryList, GmailSendError> {
        let params = GmailHistoryListParams {
            start_history_id: self.history_id.as_deref().unwrap_or_default(),
            label_id: Some(&self.mailbox),
            history_types: &[],
            max_results: None,
            page_token,
        };
        GmailHistoryList::new(&self.auth, &self.user_id, &params)
    }

    fn get_message(&self, id: &str) -> Result<GmailMessageGet, GmailSendError> {
        GmailMessageGet::new(
            &self.auth,
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

impl GmailCoroutine for GmailHistoryPoll {
    type Yield = GmailHistoryPollYield;
    type Return = Result<Infallible, GmailHistoryPollError>;

    fn resume(&mut self, bytes: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let mut bytes = bytes;
        loop {
            match mem::replace(&mut self.state, State::Done) {
                State::Baseline(mut profile) => match profile.resume(bytes.take()) {
                    GmailCoroutineState::Yielded(GmailYield::WantsRead) => {
                        self.state = State::Baseline(profile);
                        return GmailCoroutineState::Yielded(GmailHistoryPollYield::WantsRead);
                    }
                    GmailCoroutineState::Yielded(GmailYield::WantsWrite(out)) => {
                        self.state = State::Baseline(profile);
                        return GmailCoroutineState::Yielded(GmailHistoryPollYield::WantsWrite(
                            out,
                        ));
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
                    let list = match self.list_history(None) {
                        Ok(list) => list,
                        Err(err) => return GmailCoroutineState::Complete(Err(err.into())),
                    };
                    self.state = State::Listing {
                        list,
                        cycle: Cycle::default(),
                    };
                    return GmailCoroutineState::Yielded(GmailHistoryPollYield::WantsSleep(
                        Duration::from_secs(POLL_SECONDS),
                    ));
                }
                State::Listing {
                    mut list,
                    mut cycle,
                } => match list.resume(bytes.take()) {
                    GmailCoroutineState::Yielded(GmailYield::WantsRead) => {
                        self.state = State::Listing { list, cycle };
                        return GmailCoroutineState::Yielded(GmailHistoryPollYield::WantsRead);
                    }
                    GmailCoroutineState::Yielded(GmailYield::WantsWrite(out)) => {
                        self.state = State::Listing { list, cycle };
                        return GmailCoroutineState::Yielded(GmailHistoryPollYield::WantsWrite(
                            out,
                        ));
                    }
                    GmailCoroutineState::Complete(Err(err)) => {
                        if err.status() == Some(404) {
                            debug!("history cursor expired, re-baselining");
                            let profile = match GmailProfileGet::new(&self.auth, &self.user_id) {
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
                            let list = match self.list_history(Some(&token)) {
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
                            return GmailCoroutineState::Yielded(GmailHistoryPollYield::Diff(diff));
                        }

                        let ids = mem::take(&mut cycle.added_ids);
                        let current = match self.get_message(&ids[0]) {
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
                        return GmailCoroutineState::Yielded(GmailHistoryPollYield::WantsRead);
                    }
                    GmailCoroutineState::Yielded(GmailYield::WantsWrite(out)) => {
                        self.state = State::Fetching {
                            ids,
                            index,
                            current,
                            cycle,
                        };
                        return GmailCoroutineState::Yielded(GmailHistoryPollYield::WantsWrite(
                            out,
                        ));
                    }
                    GmailCoroutineState::Complete(result) => {
                        match result {
                            Ok(out) => cycle.added.push(out.response),
                            // NOTE: a just-added message may already be gone
                            // by the time we fetch it; skip it rather than
                            // tearing the watch down.
                            Err(err) => trace!("skipping message get: {err}"),
                        }

                        let index = index + 1;
                        if index < ids.len() {
                            let current = match self.get_message(&ids[index]) {
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
                            return GmailCoroutineState::Yielded(GmailHistoryPollYield::Diff(diff));
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
