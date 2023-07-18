/*
use crate::{Message, SetupClient, SetupServer};
use anyhow::Context;
use webtransport_generic::Session as Generic;

use super::{RecvControl, RecvObjects, SendControl, SendObjects};

pub struct Session<S: Generic> {
	pub send_control: SendControl<S::SendStream>,
	pub recv_control: RecvControl<S::RecvStream>,
	pub send_objects: SendObjects<S>,
	pub recv_objects: RecvObjects<S>,
}

impl<S: Generic + Send> Session<S> {
	pub async fn accept(
		session: S,
	) -> anyhow::Result<AcceptSetup<S>> {
		let send_objects = SendObjects::new(session.clone());
		let recv_objects = RecvObjects::new(session.clone());

		let setup_client = match recv_control await.context("failed to read SETUP")? {
			Message::SetupClient(setup) => setup,
			_ => anyhow::bail!("expected CLIENT SETUP"),
		};

		Ok(AcceptSetup {
			setup_client,
			control,
			objects,
		})
	}

	pub fn split(self) -> (Control<C>, Objects<C>) {
		(self.control, self.objects)
	}
}

pub struct AcceptSetup<C: Generic + Send> {
	setup_client: SetupClient,
	control: Control<C>,
	objects: Objects<C>,
}

impl<C: Generic + Send> AcceptSetup<C> {
	// Return the setup message we received.
	pub fn setup(&self) -> &SetupClient {
		&self.setup_client
	}

	// Accept the session with our own setup message.
	pub async fn accept(mut self, setup_server: SetupServer) -> anyhow::Result<Session<C>> {
		self.control.send(setup_server).await?;
		Ok(Session {
			control: self.control,
			objects: self.objects,
		})
	}

	pub async fn reject(self) -> anyhow::Result<()> {
		// TODO Close the QUIC connection with an error code.
		Ok(())
	}
}

*/
