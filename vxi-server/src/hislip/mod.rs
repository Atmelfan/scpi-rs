//! # `TCPIP::<IP address|hostname>::HISLIP`
//! HiSLIP protocol

use core::cell::RefCell;
use core::cmp::min;

use arrayvec::{Array, ArrayVec};
use log::debug;
use smoltcp::socket::{SocketHandle, SocketRef, TcpSocket, SocketSet};

use client::Client;

use crate::hislip::errors::{Error, FatalErrorCode};
use crate::hislip::messages::{Message, MessageType};
use crate::SocketPool;

pub mod errors;
pub mod messages;
pub mod client;

fn read_from_socket<'a>(socket: &'a mut SocketRef<TcpSocket>) -> Result<Message<'a>, Error> {
    socket
        .recv(|buffer| {
            let recvd_len = buffer.len();
            match Message::from_buffer(buffer) {
                // Consume message
                Ok((len, msg)) => {
                    log::debug!("Recv: {:?}", msg);
                    (len, Ok(msg))
                }
                // Invalid or incomplete message
                Err(x) => {
                    if x.message().is_some() {
                        log::error!("Error: {:?}", x);
                        (recvd_len, Err(x))
                    } else {
                        (0, Err(x))
                    }
                }
            }
        })
        .unwrap()
}

fn send_message<'a>(socket: &'a mut SocketRef<TcpSocket>, msg: Message) -> smoltcp::Result<()> {
    log::debug!("Send: {:?}", msg);
    socket.send(|buffer| msg.fill_buffer(buffer))
}

fn send_error<'a>(socket: &'a mut SocketRef<TcpSocket>, err: Error) -> smoltcp::Result<()> {
    log::error!("Error: {:?}", err);
    if matches!(err, Error::Fatal(_,_) | Error::NonFatal(_,_)) {
        socket.send(|buffer| err.message().unwrap().fill_buffer(buffer))
    } else {
        Ok(())
    }
}

pub struct Server<A: Array, C: Array> {
    port: u16,
    vendor_id: u16,
    listener: Option<SocketHandle>,
    session_ids: RefCell<u16>,
    connections: RefCell<ArrayVec<A>>,
    clients: RefCell<ArrayVec<C>>,
}

impl<A, C> Server<A, C>
where
    A: Array<Item = SocketHandle>,
    C: Array<Item = Client>,
{
    const MAX_SUPPORTED_PROTOCOL_VERSION: u16 = (2 << 8) + 0;

    pub fn new(port: u16, vendor_id: u16) -> Self {
        Server {
            port,
            vendor_id,
            session_ids: RefCell::new(0),
            listener: None,
            connections: RefCell::new(ArrayVec::new()),
            clients: RefCell::new(ArrayVec::new()),
        }
    }

    pub fn serve<X>(&mut self, sm: &mut SocketPool<X>, sockets: &mut SocketSet)
    where
        X: Array<Item = SocketHandle>,
    {
        // Listen for new connections
        if let Some(handle) = self.listener {
            let socket = sockets.get::<TcpSocket>(handle);

            if socket.is_active() {
                debug!("{:?}:{} connected", handle, socket.local_endpoint());
                self.connections.borrow_mut().push(handle);
                self.listener = None;
            }
        }

        if self.listener.is_none() {
            self.listener = sm.get_available_socket();
            if let Some(handle) = self.listener {
                log::debug!("{:?}:{} listening...", handle, self.port);
                let mut socket = sockets.get::<TcpSocket>(handle);
                socket.listen(self.port).unwrap();
            }
        }

        // Close any disconnected sockets
        self.connections.borrow_mut().retain(|handle| {

            let retain = {
                let mut socket = sockets.get::<TcpSocket>(*handle);

                debug!("Polling {:?}, rx {}, tx {}", handle, socket.may_recv(), socket.may_send());
                if socket.may_recv() {
                    match read_from_socket(&mut socket) {
                        Ok(msg) => {
                            match msg.message_type {
                                MessageType::Initialize => {
                                    debug_assert_eq!(msg.control_code, 0);
                                    let prot_version = ((msg.message_parameter & 0xFFFF0000) >> 16) as u16;
                                    let client_vendor = (msg.message_parameter & 0x0000FFFF) as u16;

                                    log::debug!("Initialize, protocol = {}, vendor = {}, sub-address = '{:?}'", prot_version, client_vendor, msg.payload);

                                    let min_version = min(Self::MAX_SUPPORTED_PROTOCOL_VERSION, prot_version) as u32;
                                    let session_id = self.session_ids.replace_with(|x| *x + 2) as u32;

                                    //TODO: Init options
                                    socket.send(|buffer| {
                                        MessageType::InitializeResponse
                                            .message_params(&[], 0,
                                                            min_version << 16 | session_id)
                                            .fill_buffer(buffer)
                                    }).unwrap();

                                    self.clients.borrow_mut().push(
                                        Client::new(session_id as u16, *handle));
                                }
                                MessageType::AsyncInitialize => {
                                    debug_assert_eq!(msg.control_code, 0);
                                    debug_assert_eq!(msg.payload.len(), 0);
                                    let session_id = (msg.message_parameter & 0x0000FFFF) as u16;
                                    if let Some(client) = self.clients.borrow_mut()
                                        .iter_mut()
                                        .find(|client|
                                            client.get_session_id() == session_id && !client.has_async_handle()) {
                                        client.set_async_handle(*handle);

                                        send_message(&mut socket, MessageType::AsyncInitializeResponse
                                            .message_params(&[], 0,self.vendor_id as u32)).unwrap();


                                    } else {
                                        send_error(&mut socket, Error::Fatal(FatalErrorCode::InvalidInitialization, b"Invalid async session id")).unwrap();
                                        socket.close();
                                    }
                                }
                                _ => {
                                    send_error(&mut socket, Error::Fatal(FatalErrorCode::InvalidInitialization, b"Invalid message before init")).unwrap();
                                    socket.close();
                                }
                            }
                            // Connection is either moved to client or closed
                            false
                        },
                        Err(x) => if let Some(msg) = x.message() {
                            log::error!("Error: {:?}", x);
                            socket.send(|buffer| msg.fill_buffer(buffer)).unwrap();
                            false
                        }else {
                            true
                        }

                    }
                } else if socket.may_send() {
                    debug!("{:?} closed", handle);
                    socket.close();
                    false
                } else {
                    debug!("{:?} no tx", handle);
                    true
                }

            };
            retain
        });

        self.clients
            .borrow_mut()
            .retain(|client| client.serve(sm, sockets).unwrap())
    }
}
