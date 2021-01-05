use arrayvec::{Array, ArrayVec};
use byteorder::{ByteOrder, NetworkEndian};
use crate::vxi11::xdr::{XdrEnum, XdrError, XdrUnpack, XdrReader, XdrPack, XdrWriter};
use smoltcp::socket::{SocketHandle, TcpSocket, UdpSocket, SocketSet};
use crate::SocketPool;
use core::cell::{RefCell, RefMut};
use smoltcp::wire::IpEndpoint;

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum AuthFlavor {
    None,
    Sys,
    Short,
    Dh,
    Gss
}

impl Default for AuthFlavor {
    fn default() -> Self {
        Self::None
    }
}

impl XdrEnum for AuthFlavor {
    fn xdr_to_discriminant(&self) -> i32 {
        match self {
            AuthFlavor::None => 0,
            AuthFlavor::Sys => 1,
            AuthFlavor::Short => 2,
            AuthFlavor::Dh => 3,
            AuthFlavor::Gss => 6,
        }
    }

    fn xdr_from_discriminant(d: i32) -> Result<Self, XdrError> {
        match d {
            0 => Ok(Self::None),
            1 => Ok(Self::Sys),
            2 => Ok(Self::Short),
            3 => Ok(Self::Dh),
            6 => Ok(Self::Gss),
            _ => Err(XdrError::InvalidDiscriminant)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum AcceptStat {
    /// Executed successfully
    Success,
    /// Remote hasn't exported program
    ProgUnavail,
    /// Remote can't support version #
    ProgMismatch{
        low: u32,
        high: u32
    },
    /// Program can't support procedure
    ProcUnavail,
    /// Procedure can't decode params
    GarbageArgs,
    /// E.g. memory allocation failure
    SystemErr
}

impl Default for AcceptStat {
    fn default() -> Self {
        Self::Success
    }
}

impl XdrEnum for AcceptStat {
    fn xdr_to_discriminant(&self) -> i32 {
        match self {
            AcceptStat::Success => 0,
            AcceptStat::ProgUnavail => 1,
            AcceptStat::ProgMismatch{ .. } => 2,
            AcceptStat::ProcUnavail => 3,
            AcceptStat::GarbageArgs => 4,
            AcceptStat::SystemErr => 5,
        }
    }

    fn xdr_from_discriminant(d: i32) -> Result<Self, XdrError> {
        match d {
            0 => Ok(Self::Success),
            1 => Ok(Self::ProgUnavail),
            2 => Ok(Self::ProgMismatch{ low: 0, high: 0 }),
            3 => Ok(Self::ProcUnavail),
            4 => Ok(Self::GarbageArgs),
            5 => Ok(Self::SystemErr),
            _ => Err(XdrError::InvalidDiscriminant)
        }
    }
}

impl XdrUnpack for AcceptStat {
    fn xdr_unpack(&mut self, reader: &mut XdrReader) -> Result<(), XdrError> {
        let mut x = AcceptStat::default();
        reader.read_enum(&mut x)?;
        match &mut x {
            AcceptStat::ProgMismatch { low, high } => {
                reader.read_u32(low)?;
                reader.read_u32(high)
            }
            _ => Ok(())
        }
    }
}

impl XdrPack for AcceptStat {
    fn xdr_pack<A>(&self, writer: &mut XdrWriter<A>) -> Result<(), XdrError> where A: Array<Item=u8> {
        writer.write_enum(*self)?;
        match self {
            AcceptStat::ProgMismatch { low, high } => {
                writer.write_u32(*low)?;
                writer.write_u32(*high)
            }
            _ => Ok(())
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum RejectStat {
    /// RPC version number != 2
    RpcMismatch,
    /// Remote can't authenticate caller
    AuthError
}

impl Default for RejectStat {
    fn default() -> Self {
        Self::RpcMismatch
    }
}

impl XdrEnum for RejectStat {
    fn xdr_to_discriminant(&self) -> i32 {
        match self {
            RejectStat::RpcMismatch => 0,
            RejectStat::AuthError => 1
        }
    }

    fn xdr_from_discriminant(d: i32) -> Result<Self, XdrError> {
        match d {
            0 => Ok(Self::RpcMismatch),
            1 => Ok(Self::AuthError),
            _ => Err(XdrError::InvalidDiscriminant)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum AuthStat {
    /// Success
    Ok,
    /// Bad credential (seal broken)
    BadCred,
    /// Client must begin new session
    RejectedCred,
    /// Bad verifier (seal broken)
    BadVerf,
    /// Verifier expired or replayed
    RejectedVerf,
    /// Rejected for security reasons
    TooWeak,
    /// Bogus response verifier
    InvalidResp,
    /// Reason unknown
    Failed,
    /// Kerberos generic error
    KerbGeneric,
    /// Time of credential expired
    TimeExpire,
    /// Problem with ticket file
    TktFile,
    /// Can't decode authenticator
    Decode,
    /// Wrong net address in ticket
    NetAddr,
    /// No credentials for user
    RpcSecGssCredProblem,
    /// Problem with context
    RpcSecGssCtxProblem
}

impl Default for AuthStat {
    fn default() -> Self {
        Self::Ok
    }
}

impl XdrEnum for AuthStat {
    fn xdr_to_discriminant(&self) -> i32 {
        match self {
            AuthStat::Ok => 0,
            AuthStat::BadCred => 1,
            AuthStat::RejectedCred => 2,
            AuthStat::BadVerf => 3,
            AuthStat::RejectedVerf => 4,
            AuthStat::TooWeak => 5,
            AuthStat::InvalidResp => 6,
            AuthStat::Failed => 7,
            AuthStat::KerbGeneric => 8,
            AuthStat::TimeExpire => 9,
            AuthStat::TktFile => 10,
            AuthStat::Decode => 11,
            AuthStat::NetAddr => 12,
            AuthStat::RpcSecGssCredProblem => 13,
            AuthStat::RpcSecGssCtxProblem => 14
        }
    }

    fn xdr_from_discriminant(d: i32) -> Result<Self, XdrError> {
        match d {
            0 => Ok(Self::Ok),
            1 => Ok(Self::BadCred),
            2 => Ok(Self::RejectedCred),
            3 => Ok(Self::BadVerf),
            4 => Ok(Self::RejectedVerf),
            5 => Ok(Self::TooWeak),
            6 => Ok(Self::InvalidResp),
            7 => Ok(Self::Failed),
            8 => Ok(Self::KerbGeneric),
            9 => Ok(Self::TimeExpire),
            10 => Ok(Self::TktFile),
            11 => Ok(Self::Decode),
            12 => Ok(Self::NetAddr),
            13 => Ok(Self::RpcSecGssCredProblem),
            14 => Ok(Self::RpcSecGssCtxProblem),
            _ => Err(XdrError::InvalidDiscriminant)
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct OpaqueAuth {
    flavor: AuthFlavor,
    //Should be 400
    body: ArrayVec<[u8; 512]>
}

impl OpaqueAuth {
    const MAXSIZE: usize = 400 + 4 + 4;
}

impl XdrUnpack for OpaqueAuth {
    fn xdr_unpack(&mut self, reader: &mut XdrReader) -> Result<(), XdrError> {
        reader.read_enum(&mut self.flavor)?;
        reader.read_variable_opaque(&mut self.body)
    }
}

impl XdrPack for OpaqueAuth {
    fn xdr_pack<A>(&self, writer: &mut XdrWriter<A>) -> Result<(), XdrError> where A: Array<Item=u8> {
        writer.write_enum(self.flavor.clone())?;
        writer.write_variable_opaque(&self.body)
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct CallBody {
    rpcvers: u32,
    prog: u32,
    vers: u32,
    proc: u32,
    cred: OpaqueAuth,
    verf: OpaqueAuth
}

impl CallBody {
    const MAXSIZE: usize = 2*OpaqueAuth::MAXSIZE + 16;
}

impl XdrUnpack for CallBody {
    fn xdr_unpack(&mut self, reader: &mut XdrReader) -> Result<(), XdrError> {
        reader.read_u32(&mut self.rpcvers)?;
        reader.read_u32(&mut self.prog)?;
        reader.read_u32(&mut self.vers)?;
        reader.read_u32(&mut self.proc)?;
        reader.read_structure(&mut self.cred)?;
        reader.read_structure(&mut self.verf)
    }
}

impl XdrPack for CallBody {
    fn xdr_pack<A>(&self, writer: &mut XdrWriter<A>) -> Result<(), XdrError> where A: Array<Item=u8> {
        writer.write_u32(self.rpcvers)?;
        writer.write_u32(self.prog)?;
        writer.write_u32(self.vers)?;
        writer.write_u32(self.proc)?;
        writer.write_structure(&self.cred)?;
        writer.write_structure(&self.verf)
    }
}


#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct AcceptedReply {
    verf: OpaqueAuth,
    stat: AcceptStat
}

impl AcceptedReply {
    const MAXSIZE: usize = OpaqueAuth::MAXSIZE + 4;
}

impl XdrUnpack for AcceptedReply {
    fn xdr_unpack(&mut self, reader: &mut XdrReader) -> Result<(), XdrError> {
        reader.read_structure(&mut self.verf)?;
        reader.read_structure(&mut self.stat)
    }
}

impl XdrPack for AcceptedReply {
    fn xdr_pack<A>(&self, writer: &mut XdrWriter<A>) -> Result<(), XdrError> where A: Array<Item=u8> {
        writer.write_structure(&self.verf)?;
        writer.write_structure(&self.stat)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RejectedReply {
    RpcMismatch {
        low: u32,
        high: u32
    },
    AuthError {
        stat: AuthStat
    }
}

impl XdrUnpack for RejectedReply {
    fn xdr_unpack(&mut self, reader: &mut XdrReader) -> Result<(), XdrError> {
        let mut x = 0;
        reader.read_i32(&mut x)?;
        match x {
            0 => {
                let mut low = 0u32;
                let mut high = 0u32;
                reader.read_u32(&mut low)?;
                reader.read_u32(&mut high)?;
                *self = RejectedReply::RpcMismatch {
                    low,
                    high
                };
                Ok(())
            },
            1 => {
                let mut stat = AuthStat::Ok;
                reader.read_enum(&mut stat)?;
                *self = RejectedReply::AuthError {
                    stat
                };
                Ok(())
            },
            _ => Err(XdrError::InvalidDiscriminant)

        }
    }
}

impl XdrPack for RejectedReply {
    fn xdr_pack<A>(&self, writer: &mut XdrWriter<A>) -> Result<(), XdrError> where A: Array<Item=u8> {
        match self {
            RejectedReply::RpcMismatch { low, high } => {
                writer.write_u32(0)?;
                writer.write_u32(*low)?;
                writer.write_u32(*high)?;
            }
            RejectedReply::AuthError { stat } => {
                writer.write_u32(0)?;
                writer.write_enum(stat.clone())?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ReplyBody {
    Accepted(AcceptedReply),
    Denied(RejectedReply)
}

impl ReplyBody {
    const MAXSIZE: usize = 4 + AcceptedReply::MAXSIZE;
}

impl Default for ReplyBody {
    fn default() -> Self {
        Self::Accepted(Default::default())
    }
}

impl XdrUnpack for ReplyBody {
    fn xdr_unpack(&mut self, reader: &mut XdrReader) -> Result<(), XdrError> {
        let mut x = 0;
        reader.read_i32(&mut x)?;
        match x {
            0 => {
                let mut accepted = AcceptedReply::default();
                reader.read_structure(&mut accepted)?;
                *self = ReplyBody::Accepted(accepted);
                Ok(())
            },
            1 => {
                let mut rejected = RejectedReply::AuthError { stat: AuthStat::Ok };
                reader.read_structure(&mut rejected)?;
                *self = ReplyBody::Denied(rejected);
                Ok(())
            }
            _ => Err(XdrError::InvalidDiscriminant)

        }
    }
}

impl XdrPack for ReplyBody {
    fn xdr_pack<A>(&self, writer: &mut XdrWriter<A>) -> Result<(), XdrError> where A: Array<Item=u8> {
        match self {
            ReplyBody::Accepted(accepted) => {
                writer.write_u32(0)?;
                writer.write_structure(accepted)?;
            }
            ReplyBody::Denied(rejected) => {
                writer.write_u32(1)?;
                writer.write_structure(rejected)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum MessageType {
    Call(CallBody),
    Reply(ReplyBody)
}

impl MessageType {
    const MAX_CALL_SIZE: usize = CallBody::MAXSIZE + 4;
    const MAX_REPLY_SIZE: usize = ReplyBody::MAXSIZE + 4;
}

impl Default for MessageType {
    fn default() -> Self {
        MessageType::Call(Default::default())
    }
}

impl XdrUnpack for MessageType {
    fn xdr_unpack(&mut self, reader: &mut XdrReader) -> Result<(), XdrError> {
        let mut x = 0;
        reader.read_i32(&mut x)?;
        match x {
            0 => {
                let mut callbody = CallBody::default();
                reader.read_structure(&mut callbody)?;
                *self = MessageType::Call(callbody);
                Ok(())
            },
            1 => {
                let mut replybody = ReplyBody::default();
                reader.read_structure(&mut replybody)?;
                *self = MessageType::Reply(replybody);
                Ok(())
            }
            _ => Err(XdrError::InvalidDiscriminant)
        }
    }
}

impl XdrPack for MessageType {
    fn xdr_pack<A>(&self, writer: &mut XdrWriter<A>) -> Result<(), XdrError> where A: Array<Item=u8> {
        match self {
            MessageType::Call(callbody) => {
                writer.write_u32(0)?;
                writer.write_structure(callbody)?;
            }
            MessageType::Reply(replybody) => {
                writer.write_u32(1)?;
                writer.write_structure(replybody)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct RpcMessage {
    xid: u32,
    mtype: MessageType
}

impl RpcMessage {
    pub const MAX_CALL_SIZE: usize = MessageType::MAX_CALL_SIZE + 4;
    pub const MAX_REPLY_SIZE: usize = MessageType::MAX_CALL_SIZE + 4;
}

impl XdrUnpack for RpcMessage {
    fn xdr_unpack(&mut self, reader: &mut XdrReader) -> Result<(), XdrError> {
        reader.read_u32(&mut self.xid)?;
        reader.read_structure(&mut self.mtype)
    }
}

impl XdrPack for RpcMessage {
    fn xdr_pack<A>(&self, writer: &mut XdrWriter<A>) -> Result<(), XdrError> where A: Array<Item=u8> {
        writer.write_u32(self.xid)?;
        writer.write_structure(&self.mtype)
    }
}

pub enum RpcHandleStatus {
    Success,
    SuccessNoReply,
    NoSuchProcedure
}

pub trait RpcHandler<R> where R: Array<Item=u8> {
    /// Called by RpcService
    fn rpc_call(&self, _proc: u32, _args: &mut XdrReader, _reply: &mut XdrWriter<R>) -> Result<RpcHandleStatus, XdrError> {
        Ok(RpcHandleStatus::NoSuchProcedure)
    }

    /// Called by RpcClient
    fn rpc_reply(&self, _reply: &mut XdrReader) -> Result<(), XdrError> {
        Ok(())
    }
}

pub struct RpcService<A, T> where
    A: Array<Item = (SocketHandle, ArrayVec<T>)>,
    T: Array<Item = u8> {
    prog: u32,
    vers: u32,
    port: u32,
    xdr_writer: RefCell<XdrWriter<T>>,
    connections: RefCell<ArrayVec<A>>,
    // TCP port
    tcp: Option<SocketHandle>,
    // UDP port
    udp: Option<SocketHandle>
}

impl<A, T> RpcService<A, T> where
    A: Array<Item = (SocketHandle, ArrayVec<T>)>,
    T: Array<Item = u8> {

    pub fn new(prog: u32, vers: u32, port: u32, tcp: Option<SocketHandle>, udp: Option<SocketHandle>) -> Self
    {
        RpcService {
            prog,
            vers,
            port,
            xdr_writer: RefCell::new(XdrWriter::new()),
            connections: RefCell::new(ArrayVec::new()),
            tcp,
            udp
        }
    }

    fn reply<R>(&self, xid: u32, reply: &mut XdrWriter<R>, replybody: ReplyBody) -> Result<(), XdrError>
        where R: Array<Item=u8> {
        let reply_msg = RpcMessage {
            xid,
            mtype: MessageType::Reply(replybody)
        };
        reply.write_structure(&reply_msg)
    }

    pub(crate) fn send_reply(&self, msg: &RpcMessage, handle: SocketHandle, sockets: &mut SocketSet) {


    }

    fn handle_message(&self, msg: &RpcMessage,
                         args: &mut XdrReader,
                         reply: &mut XdrWriter<T>,
                         handler: &dyn RpcHandler<T>) -> Result<bool, XdrError> {

        match &msg.mtype {
            MessageType::Call(callbody) => {
                reply.begin();
                // Check rpc version
                if callbody.rpcvers != 2 {
                    log::warn!("RPC({}): Bad RPC version: {}", self.prog, callbody.rpcvers);
                    self.reply(msg.xid, reply, ReplyBody::Denied(RejectedReply::RpcMismatch {
                        low: 2,
                        high: 2
                    }))?;
                    return Ok(true)
                }
                if callbody.cred.flavor != AuthFlavor::None {
                    log::warn!("RPC({}): Bad RPC version: {}", self.prog, callbody.rpcvers);
                    self.reply(msg.xid, reply, ReplyBody::Denied(RejectedReply::AuthError {
                        stat: AuthStat::Ok
                    }))?;
                    return Ok(true)
                }
                if callbody.prog != self.prog {
                    log::warn!("RPC({}): Bad program number: {}", self.prog, callbody.prog);
                    self.reply(msg.xid, reply, ReplyBody::Accepted(AcceptedReply {
                        verf: Default::default(),
                        stat: AcceptStat::ProgMismatch {high: 2, low: 2}
                    }))?;
                    return Ok(true)
                }
                if callbody.vers != self.vers {
                    log::warn!("RPC({}): Bad program version: {}", self.prog, callbody.vers);
                    self.reply(msg.xid, reply, ReplyBody::Accepted(AcceptedReply {
                        verf: Default::default(),
                        stat: AcceptStat::ProgMismatch {high: 2, low: 2}
                    }))?;
                    return Ok(true)
                }

                // Let procedure fill in return values
                self.reply(msg.xid, reply, ReplyBody::Accepted(AcceptedReply {
                    verf: Default::default(),
                    stat: AcceptStat::Success
                }))?;
                match handler.rpc_call(callbody.proc, args, reply) {
                    Ok(RpcHandleStatus::Success) => {
                        // All is ok
                        Ok(true)
                    },
                    Ok(RpcHandleStatus::SuccessNoReply) => {
                        // All is ok but don't send a reply
                        Ok(false)
                    },
                    Ok(RpcHandleStatus::NoSuchProcedure) => {
                        // No such procedure
                        reply.begin();
                        self.reply(msg.xid, reply, ReplyBody::Accepted(AcceptedReply {
                            verf: Default::default(),
                            stat: AcceptStat::ProcUnavail
                        }))?;
                        Ok(true)
                    },
                    Err(_) => {
                        // Handler failed to decode args.
                        // Or failed to encode reply but let's not think about that.
                        reply.begin();
                        self.reply(msg.xid, reply, ReplyBody::Accepted(AcceptedReply {
                            verf: Default::default(),
                            stat: AcceptStat::GarbageArgs
                        }))?;
                        Ok(true)
                    }
                }

            }
            MessageType::Reply(_) => {
                Ok(false)
            }
        }
    }

    pub fn serve<X>(&mut self, sm: &mut SocketPool<X>, sockets: &mut SocketSet, handler: &dyn RpcHandler<T>) -> smoltcp::Result<()>
        where X: Array<Item=SocketHandle>{
        let mut de_msg = RpcMessage::default();
        let mut reply = self.xdr_writer.borrow_mut();

        // TCP listener
        if let Some(handle) = self.tcp {
            let socket = sockets.get::<TcpSocket>(handle);

            if socket.is_active() {
                log::debug!("{:?} connected", socket.remote_endpoint());
                self.connections.borrow_mut().push((handle, ArrayVec::new()));
                self.tcp = None;
            }
        }

        // TCP listener connected, get new listener
        if self.tcp.is_none() {
            self.tcp = sm.get_available_socket();
            if let Some(handle) = self.tcp {
                let mut socket = sockets.get::<TcpSocket>(handle);
                // Socket might not have been closed on other end
                if !socket.is_open() {
                    socket.listen(self.port as u16)?;
                    log::debug!("tcp:{:?} listening...", socket.local_endpoint());
                } else {
                    log::debug!("socket open...");
                    sm.return_socket(handle);
                }
            }
        }

        // UDP
        if let Some(handle) = self.udp {
            let mut socket = sockets.get::<UdpSocket>(handle);
            if !socket.is_open() {
                socket.bind(self.port as u16)?;
                log::debug!("udp:{:?} bound...", socket.endpoint());
            }

            let client = match socket.recv() {
                Ok((data, endpoint)) => {
                    let mut reader = XdrReader::new(data);
                    if reader.read_structure(&mut de_msg).is_ok() {
                        log::debug!("udp: Message = ({}) {:?}", reader.get_pos(), de_msg);
                        if self.handle_message(&de_msg, &mut reader, &mut reply, handler).unwrap() {
                            Some(endpoint)
                        }else{
                            None
                        }

                    }else{
                        log::debug!("udp: Invalid message");
                        None
                    }
                }
                Err(_) => None
            };
            if let Some(endpoint) = client {
                let data = reply.finish().map_err(|_| smoltcp::Error::Unrecognized)?;
                socket.send_slice(data, endpoint)?;
            }
        }

        // TCP
        // Check all tcp sessions
        for (handle, vec) in self.connections.borrow_mut().iter_mut() {
            let mut socket = sockets.get::<TcpSocket>(*handle);

            if socket.may_recv() {

                // Assemble fragments into a complete message
                let assembled = socket
                    .recv(|buffer| {
                        // Read framing header
                        if buffer.len() < 4 {
                            return (0, Ok(false));
                        }
                        let marker = NetworkEndian::read_u32(&buffer[0..4]);
                        let len = marker & 0x7FFF_FFFF;

                        // Read rest of message if available
                        if buffer.len() < (len + 4) as usize {
                            return (0, Ok(false));
                        }

                        if vec.try_extend_from_slice(&buffer[4..4+(len as usize)]).is_ok() {
                            (len as usize + 4, Ok(marker & 0x8000_0000 > 0))
                        }else{
                            (len as usize + 4, Err(smoltcp::Error::Exhausted))
                        }
                        //log::debug!("marker, len = {}, last = {}", len, (marker & 0x8000_0000) > 0);

                    })??;

                // Deserialize message
                if assembled {
                    //log::debug!("buffer = ({}) {:?}", vec.len(), vec.as_slice());
                    let mut reader = XdrReader::new(vec.as_slice());
                    if reader.read_structure(&mut de_msg).is_ok() {
                        log::debug!("tcp: Message = {:?}", de_msg);
                        if self.handle_message(&de_msg, &mut reader, &mut reply, handler).unwrap() {
                            let data = reply.finish().map_err(|_| smoltcp::Error::Unrecognized)?;
                            let mut header = [0; 4];
                            NetworkEndian::write_u32(&mut header, 0x8000_0000 | data.len() as u32);
                            socket.send_slice(&header)?;
                            socket.send_slice(data)?;
                        }

                    }else{
                        // Ignore message
                        log::debug!("udp: Invalid Message");
                    }
                    vec.clear();
                }
            }
        }

        // Close and remove disconnected sockets
        self.connections.borrow_mut().retain(|(handle, _)| {
            let mut socket = sockets.get::<TcpSocket>(*handle);
            if !socket.is_active() {
                // Socket closed on our end
                false
            } else if socket.may_send() && !socket.may_recv(){
                // Socket disconnected at client, close
                socket.close();
                sm.return_socket(*handle);
                log::debug!("{:?} closed", handle);
                false
            }else {
                true
            }
        });

        Ok(())

    }
}

pub struct RpcTcpClient<T> where
    T: Array<Item = u8> {
    prog: u32,
    vers: u32,
    port: u32,
    xid: u32,
    active: bool,
    xdr_writer: RefCell<XdrWriter<T>>,
    // TCP port
    socket_handle: SocketHandle,
    buffer: ArrayVec<T>
}

impl<T> RpcTcpClient<T> where
    T: Array<Item = u8> {

    pub fn new(prog: u32, vers: u32, port: u32, socket: SocketHandle) -> Self
    {
        RpcTcpClient {
            prog,
            vers,
            port,
            xid: 1,
            active: false,
            xdr_writer: RefCell::new(XdrWriter::new()),
            socket_handle: socket,
            buffer: ArrayVec::new()
        }
    }

    pub fn connect<R, L>(&mut self, sockets: &mut SocketSet, remote_endpoint: R, local_endpoint: L) -> smoltcp::Result<()>
        where R: Into<IpEndpoint>, L: Into<IpEndpoint> {
        let mut socket = sockets.get::<TcpSocket>(self.socket_handle);
        socket.connect(remote_endpoint, local_endpoint)
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn call<F>(&mut self, sockets: &mut SocketSet, proc: u32, f: &mut F)-> smoltcp::Result<u32>
        where F: FnMut(&RefMut<XdrWriter<T>>) {
        self.xid += 1;

        let mut socket = sockets.get::<TcpSocket>(self.socket_handle);
        if socket.may_send() {
            let mut writer = self.xdr_writer.borrow_mut();
            writer.begin();
            let call_msg = RpcMessage {
                xid: self.xid,
                mtype: MessageType::Call(CallBody {
                    rpcvers: 2,
                    prog: self.prog,
                    vers: self.vers,
                    proc,
                    cred: Default::default(),
                    verf: Default::default()
                })
            };
            writer.write_structure(&call_msg).unwrap();
            f(&writer);
            let data = writer.finish().unwrap();
            socket.send_slice(data)?;
            Ok(self.xid)
        } else {
            Ok(0)
        }
    }

    pub fn serve<X>(&mut self, sockets: &mut SocketSet, handler: &dyn RpcHandler<T>) -> smoltcp::Result<()>
        where X: Array<Item=SocketHandle>{
        let mut de_msg = RpcMessage::default();

        let mut socket = sockets.get::<TcpSocket>(self.socket_handle);
        if socket.is_active() && !self.active {
            log::debug!("connected");
        } else if !socket.is_active() && self.active {
            log::debug!("disconnected");
        }
        self.active = socket.is_active();

        if socket.may_recv() {
            // Assemble fragments into a complete message
            let mut vec = &mut self.buffer;
            let assembled = socket
                .recv(|buffer| {
                    // Read framing header
                    if buffer.len() < 4 {
                        return (0, Ok(false));
                    }
                    let marker = NetworkEndian::read_u32(&buffer[0..4]);
                    let len = marker & 0x7FFF_FFFF;

                    // Read rest of message if available
                    if buffer.len() < (len + 4) as usize {
                        return (0, Ok(false));
                    }

                    if vec.try_extend_from_slice(&buffer[4..4+(len as usize)]).is_ok() {
                        (len as usize + 4, Ok(marker & 0x8000_0000 > 0))
                    }else{
                        (len as usize + 4, Err(smoltcp::Error::Exhausted))
                    }
                    //log::debug!("marker, len = {}, last = {}", len, (marker & 0x8000_0000) > 0);

                })??;

            // Deserialize message
            if assembled {
                //log::debug!("buffer = ({}) {:?}", vec.len(), vec.as_slice());
                let mut reader = XdrReader::new(vec.as_slice());
                if reader.read_structure(&mut de_msg).is_ok() {
                    log::debug!("tcp: Message = {:?}", de_msg);
                    match &de_msg.mtype {
                        MessageType::Reply(ReplyBody::Accepted(accept)) => {
                            match accept.stat {
                                AcceptStat::Success => {
                                    log::debug!("RPC call successful");
                                    handler.rpc_reply(&mut reader).unwrap();
                                }
                                _ => {
                                    log::debug!("RPC call failed ({:?})", accept.stat);
                                }
                            }
                        },
                        MessageType::Reply(ReplyBody::Denied(reject)) => {

                        },
                        MessageType::Call(_) => {
                            // Ignore
                        }
                    }

                }else{
                    // Ignore message
                    log::debug!("udp: Invalid Message");
                }
                vec.clear();
            }
        } else if socket.may_send() {
            log::debug!("close");
            socket.close();
        }
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_message(){
        let msg = RpcMessage {
            xid: 100,
            mtype: MessageType::Call(CallBody {
                rpcvers: 2,
                prog: 100000,
                vers: 2,
                proc: 1,
                cred: Default::default(),
                verf: Default::default()
            })
        };
        //println!("{:?}", msg);

        let mut writer: XdrWriter<[u8; 512]> = XdrWriter::new();
        writer.write_structure(&msg).unwrap();
        let serialized = writer.get_slice();

        let mut de_msg = RpcMessage::default();
        let mut reader = XdrReader::new(serialized);
        reader.read_structure(&mut de_msg).unwrap();
        assert_eq!(msg, de_msg);


    }

}




