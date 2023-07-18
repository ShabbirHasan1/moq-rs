use crate::{Decode, DecodeError, Encode, Object};
use anyhow::Context;
use bytes::{Buf, BytesMut};
use futures::StreamExt;
use futures::TryStreamExt;
use std::{
	io::Cursor,
	pin::Pin,
	task::{ready, Poll},
};
use tokio::task::JoinSet;
use webtransport_generic::{AsyncRecvStream, AsyncSendStream, AsyncSession};

// TODO support clients

pub struct SendObjects<S: AsyncSession> {
	session: S,

	// A reusable buffer for encoding messages.
	buf: BytesMut,
}

impl<S: AsyncSession> SendObjects<S> {
	pub fn new(session: S) -> Self {
		Self {
			session,
			buf: BytesMut::new(),
		}
	}

	pub async fn send(&mut self, header: Object) -> anyhow::Result<S::SendStream> {
		self.buf.clear();
		header.encode(&mut self.buf).unwrap();

		let mut stream = self.session.open_uni().await.context("failed to open uni stream")?;

		stream
			.send(&mut self.buf)
			.await
			.context("failed to send data on stream")?;

		Ok(stream)
	}
}

pub struct RecvObjects<S: AsyncSession> {
	session: S,
	objects: JoinSet<anyhow::Result<(Object, S::RecvStream)>>,
}

impl<S: AsyncSession + Clone> RecvObjects<S> {
	pub fn new(session: S) -> Self {
		let streams = futures::stream::unfold(session.clone(), |mut session| async move {
			match session.accept_uni().await {
				Ok(stream) => Some((stream, session)),
				Err(e) => None,
			}
		});

		let objects = streams.map(|mut stream| async move {});

		// Decode the object header for up to 16 streams in parallel.
		// Otherwise, a lost packet for the first chunk of a stream would block future streams.
		let objects = objects.buffer_unordered(16);

		let objects = Box::pin(objects);

		//try_buffer_unordered

		let objects = JoinSet::new();

		Self { session, objects }
	}

	pub async fn next(&mut self) -> anyhow::Result<(Object, S::RecvStream)> {
		loop {
			tokio::select! {
				res = self.objects.join_next(), if !self.objects.is_empty() => {
					return res.unwrap()?;
				},
				res = self.session.accept_uni() => {
					let stream = res?;
					self.objects.spawn(async move { Self::fetch(stream).await });
				},
			}
		}
	}

	async fn fetch(mut stream: S::RecvStream) -> anyhow::Result<(Object, S::RecvStream)> {
		let mut buf = Vec::with_capacity(64);

		loop {
			stream.recv(&mut buf).await?.context("no more data")?;
			let mut peek = Cursor::new(&buf);

			match Object::decode(&mut peek) {
				Ok(header) => return Ok((header, stream)),
				Err(DecodeError::UnexpectedEnd) => continue,
				Err(err) => return Err(err.into()),
			}
		}
	}
}
