//! Infinite polling watch coroutine built on the history API.
//!
//! Starts from the history cursor of `users.getProfile`
//! ([`GmailHistoryPoll::new`]) or from a stored one
//! ([`GmailHistoryPoll::resume_from`]), then polls `users.history.list`
//! on a timer (yielding `WantsSleep`) and emits one raw
//! `GmailHistoryDiff` per tick. An expired cursor is reported to the
//! caller, who resyncs in full.
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

/// Maximum number of `users.history.list` pages read in one tick.
pub const GMAIL_HISTORY_POLL_MAX_PAGES: usize = 1000;

const POLL_SECONDS: u64 = 30;

/// Errors that can occur during the watch.
#[derive(Debug, Error)]
pub enum GmailHistoryPollError {
    /// One of the underlying Gmail exchanges failed.
    #[error(transparent)]
    Send(#[from] GmailSendError),
    /// Neither the profile nor the caller gave a history id to start
    /// from.
    #[error("Gmail history poll has no history id to start from")]
    MissingHistoryId,
    /// The listing of one tick went past
    /// [`GMAIL_HISTORY_POLL_MAX_PAGES`].
    #[error("Gmail history listing exceeds {} pages", GMAIL_HISTORY_POLL_MAX_PAGES)]
    TooManyPages,
    /// The listing answered with the page token it was asked for.
    #[error("Gmail history listing repeats its page token")]
    RepeatedPageToken,
    /// The watch was resumed after it completed.
    #[error("Gmail history poll resumed after completion")]
    Completed,
}

/// One tick's worth of mailbox changes, Gmail-native.
///
/// Consumers translate it into their own change representation; io-gmail
/// does not interpret it further.
#[derive(Clone, Debug, Default)]
pub struct GmailHistoryDiff {
    /// The history cursor after this diff, to persist for
    /// [`GmailHistoryPoll::resume_from`].
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
    /// The history cursor expired and the watch started over from the
    /// given one: the changes before it are lost, the caller resyncs in
    /// full. The watch then goes back to sleep.
    Expired(String),
}

/// I/O-free coroutine watching a mailbox by polling `users.history.list`.
///
/// Never completes successfully (its return type is `Infallible`): it
/// yields one [`GmailHistoryDiff`] per tick, and
/// [`GmailHistoryPollYield::Expired`] once it started over from the
/// current cursor because the server reported its own as expired.
///
/// It completes on the first error, the cursor of the last diff being
/// the one to resume from.
pub struct GmailHistoryPoll {
    /// The bearer credential of the requests built from now on, to
    /// replace between ticks before the token expires.
    pub auth: HttpAuthBearer,
    state: State,
    user_id: String,
    mailbox: String,
    history_id: String,
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
        let expired = false;
        let state = State::Baseline { profile, expired };

        Ok(Self {
            auth: auth.clone(),
            state,
            user_id: user_id.into(),
            mailbox: mailbox.into(),
            history_id: String::new(),
        })
    }

    /// Builds the watch over the given mailbox label from a stored
    /// history cursor; its first tick runs at once, without sleeping.
    pub fn resume_from(
        auth: &HttpAuthBearer,
        user_id: &str,
        mailbox: &str,
        history_id: &str,
    ) -> Result<Self, GmailHistoryPollError> {
        debug!("prepare gmail poll history from a stored cursor");
        trace!("user_id: {user_id:?}");
        trace!("mailbox: {mailbox:?}");
        trace!("history_id: {history_id:?}");

        if history_id.is_empty() {
            return Err(GmailHistoryPollError::MissingHistoryId);
        }

        Ok(Self {
            auth: auth.clone(),
            state: State::Polling,
            user_id: user_id.into(),
            mailbox: mailbox.into(),
            history_id: history_id.into(),
        })
    }

    fn list_history(&self, page_token: Option<&str>) -> Result<GmailHistoryList, GmailSendError> {
        let params = GmailHistoryListParams {
            start_history_id: &self.history_id,
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
        if let Some(history_id) = cycle.new_history_id {
            self.history_id = history_id;
        }

        self.state = State::Sleeping;

        GmailHistoryDiff {
            history_id: self.history_id.clone(),
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
                State::Baseline {
                    mut profile,
                    expired,
                } => match profile.resume(bytes.take()) {
                    GmailCoroutineState::Yielded(GmailYield::WantsRead) => {
                        self.state = State::Baseline { profile, expired };
                        return GmailCoroutineState::Yielded(GmailHistoryPollYield::WantsRead);
                    }
                    GmailCoroutineState::Yielded(GmailYield::WantsWrite(out)) => {
                        self.state = State::Baseline { profile, expired };
                        return GmailCoroutineState::Yielded(GmailHistoryPollYield::WantsWrite(
                            out,
                        ));
                    }
                    GmailCoroutineState::Complete(Err(err)) => {
                        return GmailCoroutineState::Complete(Err(err.into()));
                    }
                    GmailCoroutineState::Complete(Ok(out)) => {
                        let history_id = out.response.history_id.filter(|id| !id.is_empty());

                        let Some(history_id) = history_id else {
                            let err = GmailHistoryPollError::MissingHistoryId;
                            return GmailCoroutineState::Complete(Err(err));
                        };

                        debug!("history cursor baselined");
                        trace!("history_id: {history_id:?}");

                        self.history_id = history_id;
                        self.state = State::Sleeping;

                        if expired {
                            let history_id = self.history_id.clone();
                            return GmailCoroutineState::Yielded(GmailHistoryPollYield::Expired(
                                history_id,
                            ));
                        }
                    }
                },
                State::Sleeping => {
                    self.state = State::Polling;
                    return GmailCoroutineState::Yielded(GmailHistoryPollYield::WantsSleep(
                        Duration::from_secs(POLL_SECONDS),
                    ));
                }
                State::Polling => {
                    let list = match self.list_history(None) {
                        Ok(list) => list,
                        Err(err) => return GmailCoroutineState::Complete(Err(err.into())),
                    };
                    self.state = State::Listing {
                        list,
                        cycle: Cycle::default(),
                    };
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
                        if err.is_history_expired() {
                            debug!("history cursor expired, re-baselining");
                            let profile = match GmailProfileGet::new(&self.auth, &self.user_id) {
                                Ok(profile) => profile,
                                Err(err) => {
                                    return GmailCoroutineState::Complete(Err(err.into()));
                                }
                            };
                            let expired = true;
                            self.state = State::Baseline { profile, expired };
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
                            cycle.pages += 1;

                            if cycle.page_token.as_deref() == Some(token.as_str()) {
                                let err = GmailHistoryPollError::RepeatedPageToken;
                                return GmailCoroutineState::Complete(Err(err));
                            }

                            if cycle.pages == GMAIL_HISTORY_POLL_MAX_PAGES {
                                let err = GmailHistoryPollError::TooManyPages;
                                return GmailCoroutineState::Complete(Err(err));
                            }

                            let list = match self.list_history(Some(&token)) {
                                Ok(list) => list,
                                Err(err) => {
                                    return GmailCoroutineState::Complete(Err(err.into()));
                                }
                            };
                            cycle.page_token = Some(token);
                            self.state = State::Listing { list, cycle };
                            continue;
                        }

                        cycle.new_history_id = response.history_id.filter(|id| !id.is_empty());

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
                            // NOTE: a message deleted since its listing is
                            // skipped, any other error ends the watch.
                            Err(err) if err.is_not_found() => {
                                trace!("skip message gone since its listing: {err}");
                            }
                            Err(err) => return GmailCoroutineState::Complete(Err(err.into())),
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
                State::Done => {
                    let err = GmailHistoryPollError::Completed;
                    return GmailCoroutineState::Complete(Err(err));
                }
            }
        }
    }
}

#[derive(Default)]
struct Cycle {
    pages: usize,
    page_token: Option<String>,
    added_ids: Vec<String>,
    added: Vec<GmailMessage>,
    removed: Vec<GmailMessageId>,
    labels_added: Vec<GmailHistoryLabel>,
    labels_removed: Vec<GmailHistoryLabel>,
    new_history_id: Option<String>,
}

enum State {
    Baseline {
        profile: GmailProfileGet,
        expired: bool,
    },
    Sleeping,
    Polling,
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
