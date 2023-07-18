use crate::{Decode, DecodeError, Encode, Message};
use webtransport_generic::{RecvStream, SendStream};

use bytes::{Buf, BytesMut};

use std::future;
use std::io::Cursor;
use std::sync::Arc;
use tokio::sync::Mutex;

use anyhow::Context;

pub struct SendControl<S: SendStream> {
	stream: S,
	buf: BytesMut, // reuse a buffer to encode messages.
}

impl<S: SendStream> SendControl<S> {
	pub fn new(stream: S) -> Self {
		Self {
			buf: BytesMut::new(),
			stream,
		}
	}

	pub async fn send<T: Into<Message>>(&mut self, msg: T) -> anyhow::Result<()> {
		let msg = msg.into();
		log::info!("sending message: {:?}", msg);

		self.buf.clear();
		msg.encode(&mut self.buf)?;

		// TODO make this work with select!
		self.stream
			.send(&mut self.buf)
			.await
			.context("error sending control message")?;

		Ok(())
	}

	// Helper that lets multiple threads send control messages.
	pub fn share(self) -> SharedControl<S> {
		SharedControl {
			stream: Arc::new(Mutex::new(self)),
		}
	}
}

// Helper that allows multiple threads to send control messages.
// There's no equivalent for receiving since only one thread should be receiving at a time.
#[derive(Clone)]
pub struct SharedControl<S: SendStream> {
	stream: Arc<Mutex<SendControl<S>>>,
}

impl<S: SendStream> SharedControl<S> {
	pub async fn send<T: Into<Message>>(&mut self, msg: T) -> anyhow::Result<()> {
		let mut stream = self.stream.lock().await;
		stream.send(msg).await
	}
}

pub struct RecvControl<R: RecvStream> {
	stream: R,
	buf: BytesMut, // data we've read but haven't fully decoded yet
}

impl<R: RecvStream> RecvControl<R> {
	pub fn new(stream: R) -> Self {
		Self {
			buf: BytesMut::new(),
			stream,
		}
	}

	// Read the next full message from the stream.
	pub async fn recv(&mut self) -> anyhow::Result<Message> {
		loop {
			// Read the contents of the buffer
			let mut peek = Cursor::new(&self.buf);

			match Message::decode(&mut peek) {
				Ok(msg) => {
					// We've successfully decoded a message, so we can advance the buffer.
					self.buf.advance(peek.position() as usize);

					log::info!("received message: {:?}", msg);
					return Ok(msg);
				}
				Err(DecodeError::UnexpectedEnd) => {
					// The decode failed, so we need to append more data.
					future::poll_fn(|cx| self.stream.poll_recv(cx, &mut self.buf))
						.await
						.context("error receiving control message")?;
				}
				Err(e) => return Err(e.into()),
			}
		}
	}
}
