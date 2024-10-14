use crate::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) conf: Arc<Conf>,
}
