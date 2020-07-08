use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{
    io::{AsyncRead, AsyncWrite},
    stream::Stream,
};

use crate::peer::Message;

struct PeerConnection<S: AsyncRead + AsyncWrite> {
    stream: S,
}

impl<S: AsyncRead + AsyncWrite> PeerConnection<S> {
    pub fn new(stream: S) -> Self {
        Self { stream }
    }
}

impl<S: AsyncRead + AsyncWrite> Stream for PeerConnection<S> {
    type Item = Message;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        todo!()
    }
}
