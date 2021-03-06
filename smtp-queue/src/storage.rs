use bytes::BytesMut;
use tokio::prelude::*;

use mail::{FoundInflightMail, InflightMail, Mail, QueuedMail};

// TODO: (B) replace all these Box by impl Trait syntax hide:impl-trait-in-trait
// TODO: (B) for a clean api, the futures should not take ownership and return
// but rather take a reference (when async/await will be done)
pub trait Storage<M, QM: QueuedMail<M>, IM: InflightMail<M>, FIM: FoundInflightMail<M>>:
    Sized + Send + Sync + 'static
{
    fn list_queue(&self) -> Box<dyn Stream<Item = QM, Error = ()>>;
    fn find_inflight(&self) -> Box<dyn Stream<Item = FIM, Error = ()>>;

    fn cancel_found_inflight(&self, mail: FIM) -> Box<dyn Future<Item = QM, Error = ()> + Send>;

    fn enqueue<S>(&self, mail: Mail<S, M>) -> Box<dyn Future<Item = QM, Error = ()>>
    where
        S: Stream<Item = BytesMut, Error = ()>;

    fn send_start(&self, mail: QM) -> Box<dyn Future<Item = IM, Error = ()> + Send>;
    fn send_done(&self, mail: IM) -> Box<dyn Future<Item = (), Error = ()> + Send>;
    fn send_cancel(&self, mail: IM) -> Box<dyn Future<Item = QM, Error = ()> + Send>;
}
