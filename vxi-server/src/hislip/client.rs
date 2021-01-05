use arrayvec::Array;
use byteorder::{NetworkEndian, ByteOrder};
use smoltcp::socket::{TcpSocket, SocketHandle, SocketSet};

use crate::{hislip, SocketPool};
use crate::hislip::errors::{Error, FatalErrorCode, NonFatalErrorCode};
use crate::hislip::messages::MessageType;

pub struct Client {
    session_id: u16,
    sync_handle: SocketHandle,
    async_handle: Option<SocketHandle>,
    client_max_message: u64,
    server_max_message: u64,
}

impl Client {
    pub(crate) fn new(session_id: u16, sync_handle: SocketHandle) -> Client {
        Client {
            session_id,
            sync_handle,
            async_handle: None,
            client_max_message: 256,
            server_max_message: 256,
        }
    }

    pub fn get_session_id(&self) -> u16 {
        self.session_id
    }

    pub fn set_async_handle(&mut self, handle: SocketHandle) {
        self.async_handle = Some(handle);
    }

    pub fn has_async_handle(&self) -> bool {
        self.async_handle.is_some()
    }

    fn close<A>(&mut self, sm: &mut SocketPool<A>, sockets: &mut SocketSet)
    where
        A: Array<Item=SocketHandle>,{
        log::error!("Closing client {}", self.session_id);

        {
            let mut sync_socket = sockets.get::<TcpSocket>(self.sync_handle);
            sync_socket.close();
            sm.return_socket(self.sync_handle);
        }
        if let Some(async_handle) = self.async_handle {
            let mut async_socket = sockets.get::<TcpSocket>(async_handle);
            async_socket.close();
            sm.return_socket(async_handle);
        }
    }

    pub fn serve<A>(&mut self, sm: &mut SocketPool<A>, sockets: &mut SocketSet) -> Result<bool, Error>
    where
        A: Array<Item=SocketHandle>,
    {
        let mut retain = true;

        retain &= {
            let mut sync_socket = sockets.get::<TcpSocket>(self.sync_handle);
            log::debug!("Polling {:?}, rx {}, tx {}", self.sync_handle, sync_socket.may_recv(), sync_socket.may_send());
            if sync_socket.may_recv() {
                match hislip::read_from_socket(&mut sync_socket) {
                    Ok(msg) => {
                        if self.async_handle.is_none() {
                            hislip::send_error(
                                &mut sync_socket,
                                Error::Fatal(
                                    FatalErrorCode::AttemptUseWithoutBothChannels,
                                    b"Async not initialized",
                                ),
                            )
                            .unwrap();
                            sync_socket.close();
                            sm.return_socket(self.sync_handle);
                        } else {

                            match msg.message_type {
                                _ => {
                                    hislip::send_error(
                                        &mut sync_socket,
                                        Error::NonFatal(
                                            NonFatalErrorCode::UnrecognizedMessageType,
                                            b"Unrecognized message type",
                                        ),
                                    ).unwrap()
                                }
                            }
                        }
                        true
                    }
                    Err(err) => match err {
                        Error::Fatal(_, _) => {
                            hislip::send_error(&mut sync_socket, err).unwrap();
                            false
                        }
                        Error::NonFatal(_, _) => {
                            hislip::send_error(&mut sync_socket, err).unwrap();
                            true
                        },
                        _ => true
                    },
                }
            } else if sync_socket.may_send() {
                false
            } else {
                log::debug!("{:?} no tx", self.sync_handle);
                true
            }
        };

        retain &= if let Some(async_handle) = self.async_handle {
            let mut async_socket = sockets.get::<TcpSocket>(async_handle);
            log::debug!("Polling {:?}, rx {}, tx {}", async_handle, async_socket.may_recv(), async_socket.may_send());
            if async_socket.may_recv() {
                match hislip::read_from_socket(&mut async_socket) {
                    Ok(msg) => {

                        match msg.message_type {
                            MessageType::AsyncMaximumMessageSize => {
                                self.client_max_message = NetworkEndian::read_u64(msg.payload);
                                let mut payload = [0u8; 8];
                                NetworkEndian::write_u64(&mut payload, self.server_max_message);
                                hislip::send_message(
                                    &mut async_socket,
                                    MessageType::AsyncMaximumMessageSizeResponse.message(&payload),
                                )
                                .unwrap();
                            }
                            _ => {
                                hislip::send_error(
                                    &mut async_socket,
                                    Error::NonFatal(
                                        NonFatalErrorCode::UnrecognizedMessageType,
                                        b"Unrecognized message type",
                                    ),
                                )
                                .unwrap();
                            }
                        }
                        true
                    }
                    Err(err) => match err {
                        Error::Fatal(_, _) => {
                            hislip::send_error(&mut async_socket, err).unwrap();
                            false
                        }
                        Error::NonFatal(_, _) => {
                            hislip::send_error(&mut async_socket, err).unwrap();
                            true
                        },
                        _ => true
                    },
                }
            } else if async_socket.may_send() {
                false
            } else {
                log::debug!("{:?} no tx", self.sync_handle);
                true
            }
        } else {
            true
        };

        if !retain {
            self.close(sm, sockets);
        }
        Ok(retain)
    }
}
