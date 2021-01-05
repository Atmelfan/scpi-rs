use arrayvec::{Array, ArrayVec};
use smoltcp::socket::{SocketHandle, TcpSocket, SocketSet};
use core::cell::{RefCell, Cell};
use crate::vxi11::rpc::{RpcService, RpcHandler, RpcHandleStatus, RpcMessage};
use crate::vxi11::xdr::{XdrError, XdrReader, XdrWriter, XdrEnum, XdrUnpack, XdrPack};
use crate::SocketPool;

type DeviceLink = u32;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DeviceErrorCode {
    NoError,
    SyntaxError,
    DeviceNotAccessible,
    InvalidLinkIdentifier,
    ParameterError,
    ChannelNotEstablished,
    OperationNotPermitted,
    OutOfResources,
    DeviceLockByAnotherLink,
    NoLockHeldByThisLink,
    IOTimeout,
    IOError,
    InvalidAddress,
    Abort,
    ChannelAlreadyEstablished
}

impl Default for DeviceErrorCode {
    fn default() -> Self {
        Self::NoError
    }
}

impl From<DeviceError> for DeviceErrorCode {
    fn from(de: DeviceError) -> Self {
        de.error
    }
}

impl XdrEnum for DeviceErrorCode {
    fn xdr_to_discriminant(&self) -> i32 {
        match self {
            DeviceErrorCode::NoError => 0,
            DeviceErrorCode::SyntaxError => 1,
            DeviceErrorCode::DeviceNotAccessible => 3,
            DeviceErrorCode::InvalidLinkIdentifier => 4,
            DeviceErrorCode::ParameterError => 5,
            DeviceErrorCode::ChannelNotEstablished => 6,
            DeviceErrorCode::OperationNotPermitted => 8,
            DeviceErrorCode::OutOfResources => 9,
            DeviceErrorCode::DeviceLockByAnotherLink => 11,
            DeviceErrorCode::NoLockHeldByThisLink => 12,
            DeviceErrorCode::IOTimeout => 15,
            DeviceErrorCode::IOError => 17,
            DeviceErrorCode::InvalidAddress => 21,
            DeviceErrorCode::Abort => 23,
            DeviceErrorCode::ChannelAlreadyEstablished => 29
        }
    }

    fn xdr_from_discriminant(d: i32) -> Result<Self, XdrError> {
        match d {
            0 => Ok(Self::NoError),
            1 => Ok(Self::SyntaxError),
            3 => Ok(Self::DeviceNotAccessible),
            4 => Ok(Self::InvalidLinkIdentifier),
            5 => Ok(Self::ParameterError),
            6 => Ok(Self::ChannelNotEstablished),
            8 => Ok(Self::OperationNotPermitted),
            9 => Ok(Self::OutOfResources),
            11 => Ok(Self::DeviceLockByAnotherLink),
            12 => Ok(Self::NoLockHeldByThisLink),
            15 => Ok(Self::IOTimeout),
            17 => Ok(Self::IOError),
            21 => Ok(Self::InvalidAddress),
            23 => Ok(Self::Abort),
            29 => Ok(Self::ChannelAlreadyEstablished),
            _ => Err(XdrError::InvalidDiscriminant)
        }
    }
}

#[derive(Debug, Default)]
pub struct DeviceError {
    error: DeviceErrorCode
}

impl From<DeviceErrorCode> for DeviceError {
    fn from(dec: DeviceErrorCode) -> Self {
        DeviceError {
            error: dec
        }
    }
}

impl XdrUnpack for DeviceError {
    fn xdr_unpack(&mut self, reader: &mut XdrReader) -> Result<(), XdrError> {
        reader.read_enum(&mut self.error)
    }
}

impl XdrPack for DeviceError {
    fn xdr_pack<A>(&self, writer: &mut XdrWriter<A>) -> Result<(), XdrError> where A: Array<Item=u8> {
        writer.write_enum(self.error)
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct DeviceFlags {
    termcharset: bool,
    end: bool,
    waitlock: bool
}

impl XdrPack for DeviceFlags {
    fn xdr_pack<A>(&self, writer: &mut XdrWriter<A>) -> Result<(), XdrError> where A: Array<Item=u8> {
        let flags: u32 = (self.termcharset as u32) << 7 |
            (self.end as u32) << 3 |
            (self.waitlock as u32);
        writer.write_u32(flags)
    }
}

impl XdrUnpack for DeviceFlags {
    fn xdr_unpack(&mut self, reader: &mut XdrReader) -> Result<(), XdrError> {
        let mut flags = 0;
        reader.read_u32(&mut flags)?;
        self.termcharset = flags & (1 << 7) != 0;
        self.end = flags & (1 << 3) != 0;
        self.waitlock = flags & (1 << 0) != 0;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct DeviceGenericParams {
    lid: DeviceLink,
    flags: DeviceFlags,
    lock_timeout: u32,
    io_timeout: u32,
}

impl XdrUnpack for DeviceGenericParams {
    fn xdr_unpack(&mut self, reader: &mut XdrReader) -> Result<(), XdrError> {
        reader.read_u32(&mut self.lid)?;
        reader.read_structure(&mut self.flags)?;
        reader.read_u32(&mut self.lock_timeout)?;
        reader.read_u32(&mut self.io_timeout)
    }
}

#[derive(Debug, Default)]
pub struct CreateLinkParms {
    client_id: u32,
    lock_device: bool,
    lock_timeout: u32,
    device: ArrayVec<[u8; 256]>
}

impl XdrUnpack for CreateLinkParms {
    fn xdr_unpack(&mut self, reader: &mut XdrReader) -> Result<(), XdrError> {
        reader.read_u32(&mut self.client_id)?;
        reader.read_bool(&mut self.lock_device)?;
        reader.read_u32(&mut self.lock_timeout)?;
        reader.read_variable_opaque(&mut self.device)
    }
}

pub struct CreateLinkResp {
    error: DeviceErrorCode,
    lid: DeviceLink,
    abort_port: u32,
    max_recv_size: u32
}

impl XdrPack for CreateLinkResp {
    fn xdr_pack<A>(&self, writer: &mut XdrWriter<A>) -> Result<(), XdrError> where A: Array<Item=u8> {
        writer.write_enum(self.error)?;
        writer.write_u32(self.lid)?;
        writer.write_u32(self.abort_port)?;
        writer.write_u32(self.max_recv_size)
    }
}

#[derive(Debug, Default, Clone)]
pub struct DeviceWriteParms {
    lid: DeviceLink,
    io_timeout: u32,
    lock_timeout: u32,
    flags: DeviceFlags,
    //data: ArrayVec<X>
}

impl DeviceWriteParms {
    const HEADERSIZE: usize = 20;
}

impl XdrUnpack for DeviceWriteParms{
    fn xdr_unpack(&mut self, reader: &mut XdrReader) -> Result<(), XdrError> {
        reader.read_u32(&mut self.lid)?;
        reader.read_u32(&mut self.io_timeout)?;
        reader.read_u32(&mut self.lock_timeout)?;
        reader.read_structure(&mut self.flags)
        // Handling this in caller avoids a buffer
        //reader.read_variable_opaque(&mut self.data)
    }
}

#[derive(Default)]
pub struct DeviceWriteResp {
    error: DeviceErrorCode,
    size: u32
}

impl XdrPack for DeviceWriteResp {
    fn xdr_pack<A>(&self, writer: &mut XdrWriter<A>) -> Result<(), XdrError> where A: Array<Item=u8> {
        writer.write_enum(self.error)?;
        writer.write_u32(self.size)
    }
}

#[derive(Debug, Default)]
pub struct DeviceReadParms {
    lid: DeviceLink,
    request_size: u32,
    io_timeout: u32,
    lock_timeout: u32,
    flags: DeviceFlags,
    term_char: u32
}

impl XdrUnpack for DeviceReadParms {
    fn xdr_unpack(&mut self, reader: &mut XdrReader) -> Result<(), XdrError> {
        reader.read_u32(&mut self.lid)?;
        reader.read_u32(&mut self.request_size)?;
        reader.read_u32(&mut self.io_timeout)?;
        reader.read_u32(&mut self.lock_timeout)?;
        reader.read_structure(&mut self.flags)?;
        reader.read_u32(&mut self.term_char)
    }
}

pub struct DeviceReadResp {
    error: DeviceErrorCode,
    reason: u32,
    //data: ArrayVec<[u8; 256]>
}

impl DeviceReadResp {
    const HEADERSIZE: usize = 6;
}

impl XdrPack for DeviceReadResp {
    fn xdr_pack<A>(&self, writer: &mut XdrWriter<A>) -> Result<(), XdrError> where A: Array<Item=u8> {
        writer.write_enum(self.error)?;
        writer.write_u32(self.reason)
        //writer.write_variable_opaque(&self.data)
    }
}

pub struct DeviceReadStbResp {
    error: DeviceErrorCode,
    stb: u32
}

impl XdrPack for DeviceReadStbResp {
    fn xdr_pack<A>(&self, writer: &mut XdrWriter<A>) -> Result<(), XdrError> where A: Array<Item=u8> {
        writer.write_enum(self.error)?;
        writer.write_u32(self.stb)
    }
}

#[derive(Debug, Default)]
pub struct DeviceLockParms {
    lid: DeviceLink,
    flags: DeviceFlags,
    lock_timeout: u32,
}

impl XdrUnpack for DeviceLockParms {
    fn xdr_unpack(&mut self, reader: &mut XdrReader) -> Result<(), XdrError> {
        reader.read_u32(&mut self.lid)?;
        reader.read_structure(&mut self.flags)?;
        reader.read_u32(&mut self.lock_timeout)
    }
}

#[derive(Debug)]
pub enum DeviceAddrFamily {
    Tcp, //0
    Udp //1
}

impl Default for DeviceAddrFamily {
    fn default() -> Self {
        Self::Tcp
    }
}

impl XdrEnum for DeviceAddrFamily {
    fn xdr_to_discriminant(&self) -> i32 {
        match self {
            DeviceAddrFamily::Tcp => 0,
            DeviceAddrFamily::Udp => 1,
        }
    }

    fn xdr_from_discriminant(d: i32) -> Result<Self, XdrError> {
        match d {
            0 => Ok(Self::Tcp),
            1 => Ok(Self::Udp),
            _ => Err(XdrError::InvalidDiscriminant)
        }
    }
}

#[derive(Debug, Default)]
pub struct DeviceRemoteFunc {
    host_addr: u32,
    host_port: u32,
    prog_num: u32,
    prog_vers: u32,
    prog_family: DeviceAddrFamily
}

impl XdrUnpack for DeviceRemoteFunc {
    fn xdr_unpack(&mut self, reader: &mut XdrReader) -> Result<(), XdrError> {
        reader.read_u32(&mut self.host_addr)?;
        reader.read_u32(&mut self.host_port)?;
        reader.read_u32(&mut self.prog_num)?;
        reader.read_u32(&mut self.prog_vers)?;
        reader.read_enum(&mut self.prog_family)
    }
}

#[derive(Debug, Default)]
pub struct DeviceEnableSrqParms {
    lid: DeviceLink,
    enable: bool,
    handle: ArrayVec<[u8; 40]>
}

impl XdrUnpack for DeviceEnableSrqParms {
    fn xdr_unpack(&mut self, reader: &mut XdrReader) -> Result<(), XdrError> {
        reader.read_u32(&mut self.lid)?;
        reader.read_bool(&mut self.enable)?;
        reader.read_variable_opaque(&mut self.handle)
    }
}

#[derive(Debug, Default)]
pub struct DeviceDocmdParms {
    lid: DeviceLink,
    flags: DeviceFlags,
    io_timeout: u32,
    lock_timeout: u32,
    cmd: i32,
    network_order: bool,
    datasize: i32,
    data_in: ArrayVec<[u8; 256]>
}

#[derive(Debug, Default)]
pub struct DeviceSrqParms {
    handle: ArrayVec<[u8; 40]>
}

impl XdrPack for DeviceSrqParms {
    fn xdr_pack<A>(&self, writer: &mut XdrWriter<A>) -> Result<(), XdrError> where A: Array<Item=u8> {
        writer.write_variable_opaque(&self.handle)
    }
}

#[derive(Debug, Default)]
pub struct Link<BUF>
    where BUF: Array<Item = u8>{
    client_id: u32,
    lid: DeviceLink,
    device: &'static [u8],
    //In/out buffers
    input_buffer: ArrayVec<BUF>,
    output_buffer: ArrayVec<BUF>,

}

/// Number of bytes used in a write header
pub const NETBUFFER_WRITE_OVERHEAD: usize = RpcMessage::MAX_CALL_SIZE + DeviceWriteParms::HEADERSIZE;
/// Number of bytes used in a read header
pub const NETBUFFER_READ_OVERHEAD: usize = RpcMessage::MAX_REPLY_SIZE + DeviceReadResp::HEADERSIZE;

/// Create
#[macro_export]
macro_rules! vxi11_service {
    ($num_clients:expr, $net_buffer:expr, $link_buffer:expr) => {
        Vxi11Server<
            [(SocketHandle,
                ArrayVec<[u8; $net_buffer]>);
                2 + 2*($num_clients)],
            [u8; $net_buffer],
            [Link<[u8; $link_buffer]>; $num_clients],
            [u8; $link_buffer]>
    };
}


///
///
/// Generic params:
/// * `CONNECTIONS` - Array of sockethandles with TX buffer
/// * `NETBUFFER` - Array for RX buffer
///
pub struct Vxi11Server<CONNECTIONS, NETBUFFER, LINKS, LINKBUFFER> where
    CONNECTIONS: Array<Item = (SocketHandle, ArrayVec<NETBUFFER>)>,
    NETBUFFER: Array<Item = u8>,
    LINKS: Array<Item = Link<LINKBUFFER>>,
    LINKBUFFER: Array<Item = u8>{
    // RPC server for core channel
    core_rpc: RefCell<RpcService<CONNECTIONS, NETBUFFER>>,
    // RPC server for abort channel
    abort_rpc: RefCell<RpcService<CONNECTIONS, NETBUFFER>>,

    link_ids: Cell<u32>,
    links: RefCell<ArrayVec<LINKS>>
}

impl<A, T, L, X> Vxi11Server<A, T, L, X> where
    A: Array<Item = (SocketHandle, ArrayVec<T>)>,
    T: Array<Item = u8>,
    L: Array<Item = Link<X>>,
    X: Array<Item = u8>{

    // * Abort channel
    const DEVICE_ASYNC: u32 = 0x0607B0;
    const DEVICE_ASYNC_VERSION: u32 = 1;

    // Procedures
    const PROC_DEVICE_ABORT: u32 = 1;

    // * Core channel
    const DEVICE_CORE: u32 = 0x0607AF;
    const DEVICE_CORE_VERSION: u32 = 1;

    // Procedures
    const PROC_CREATE_LINK: u32 = 10;
    const PROC_DEVICE_WRITE: u32 = 11;
    const PROC_DEVICE_READ: u32 = 12;
    const PROC_DEVICE_READ_STB: u32 = 13;
    const PROC_DEVICE_TRIGGER: u32 = 14;
    const PROC_DEVICE_CLEAR: u32 = 15;
    const PROC_DEVICE_REMOTE: u32 = 16;
    const PROC_DEVICE_LOCAL: u32 = 17;
    const PROC_DEVICE_LOCK: u32 = 18;
    const PROC_DEVICE_UNLOCK: u32 = 19;
    const PROC_DEVICE_ENABLE_SRQ: u32 = 20;
    const PROC_DEVICE_DOCMD: u32 = 22;
    const PROC_DESTROY_LINK: u32 = 23;
    const PROC_CREATE_INTR_CHAN: u32 = 25;
    const PROC_DESTROY_INTR_CHAN: u32 = 26;

    // * Interrupt channel
    const DEVICE_INTR: u32 = 0x0607B1;
    const DEVICE_INTR_VERSION: u32 = 1;

    // Procedures
    const PROC_DEVICE_INTR_SRQ: u32 = 30;

    pub fn new(core: Option<SocketHandle>, abort: Option<SocketHandle>) -> Self {
        log::debug!("Vxi service, max write size={}",(T::CAPACITY - RpcMessage::MAX_CALL_SIZE - DeviceWriteParms::HEADERSIZE));
        Vxi11Server {
            core_rpc: RefCell::new(RpcService::new(
                Self::DEVICE_CORE,
                Self::DEVICE_CORE_VERSION,
                1024,
                core, None)),
            abort_rpc: RefCell::new(RpcService::new(
                Self::DEVICE_ASYNC,
                Self::DEVICE_ASYNC_VERSION,
                1025,
                abort, None)),
            link_ids: Cell::new(1),
            links: RefCell::new(ArrayVec::default())
        }
    }

    pub fn serve<C>(&mut self, sm: &mut SocketPool<C>, sockets: &mut SocketSet) -> smoltcp::Result<()>
        where C: Array<Item=SocketHandle> {
        //log::debug!("CORE");
        self.core_rpc.borrow_mut().serve(sm, sockets, self)?;
        //log::debug!("ABORT");
        self.abort_rpc.borrow_mut().serve(sm, sockets, self)
    }

    pub fn create_link(&self, client_id: u32, device: &'static [u8]) -> Result<DeviceLink, DeviceError> {
        let lid = self.link_ids.get();
        self.link_ids.set(lid + 1);
        let link = Link {
            client_id,
            lid,
            device,
            input_buffer: ArrayVec::new(),
            output_buffer: ArrayVec::new()
        };
        self.links.borrow_mut().try_push(link)
            .map(|_| lid)
            .map_err(|_| DeviceErrorCode::OutOfResources.into())
    }
}

impl<A, T, L, X> RpcHandler<T> for Vxi11Server<A, T, L, X>  where
    A: Array<Item = (SocketHandle, ArrayVec<T>)>,
    T: Array<Item = u8>,
    L: Array<Item = Link<X>>,
    X: Array<Item = u8>{
    fn rpc_call(&self, _proc: u32, _args: &mut XdrReader, _reply: &mut XdrWriter<T>) -> Result<RpcHandleStatus, XdrError> {
        match _proc {
            Self::PROC_DEVICE_ABORT => {
                let mut device_link: DeviceLink = 0;
                _args.read_u32(&mut device_link)?;
                log::debug!("PROC_DEVICE_ABORT({})", device_link);

                Ok(RpcHandleStatus::Success)
            },
            Self::PROC_CREATE_LINK => {
                let mut linkparms = CreateLinkParms::default();
                _args.read_structure(&mut linkparms)?;
                log::debug!("PROC_CREATE_LINK({:?})", linkparms);

                let mut linkresp = CreateLinkResp {
                    error: DeviceErrorCode::NoError,
                    lid: 0,
                    abort_port: 1025,
                    max_recv_size: (T::CAPACITY - RpcMessage::MAX_CALL_SIZE - DeviceWriteParms::HEADERSIZE) as u32
                };

                match self.create_link(linkparms.client_id,b"inst0") {
                    Ok(lid) => {
                        linkresp.lid = lid;
                    }
                    Err(err) => {
                        linkresp.error = err.into();
                    }
                }
                _reply.write_structure(&linkresp)?;
                Ok(RpcHandleStatus::Success)
            },
            Self::PROC_DEVICE_WRITE => {
                let mut writeparms = DeviceWriteParms::default();
                _args.read_structure(&mut writeparms)?;
                let data = _args.read_variable_opaque_slice()?;
                log::debug!("PROC_DEVICE_WRITE({:?}, {:?})", writeparms, data);
                let writeresp = DeviceWriteResp {
                    error: Default::default(),
                    size: data.len() as u32
                };
                _reply.write_structure(&writeresp)?;
                Ok(RpcHandleStatus::Success)
            },
            Self::PROC_DEVICE_READ => {
                log::debug!("PROC_DEVICE_READ");
                Ok(RpcHandleStatus::Success)
            },
            Self::PROC_DEVICE_READ_STB => {
                log::debug!("PROC_DEVICE_READ_STB");
                Ok(RpcHandleStatus::Success)
            },
            Self::PROC_DEVICE_TRIGGER => {
                log::debug!("PROC_DEVICE_TRIGGER");
                Ok(RpcHandleStatus::Success)
            },
            Self::PROC_DEVICE_CLEAR => {
                log::debug!("PROC_DEVICE_CLEAR");
                Ok(RpcHandleStatus::Success)
            },
            Self::PROC_DEVICE_REMOTE => {
                log::debug!("PROC_DEVICE_REMOTE");
                Ok(RpcHandleStatus::Success)
            },
            Self::PROC_DEVICE_LOCAL => {
                log::debug!("PROC_DEVICE_LOCAL");
                Ok(RpcHandleStatus::Success)
            },
            Self::PROC_DEVICE_LOCK => {
                log::debug!("PROC_DEVICE_LOCK");
                Ok(RpcHandleStatus::Success)
            },
            Self::PROC_DEVICE_UNLOCK => {
                log::debug!("PROC_DEVICE_UNLOCK");
                Ok(RpcHandleStatus::Success)
            },
            Self::PROC_DEVICE_ENABLE_SRQ => {
                log::debug!("PROC_DEVICE_ENABLE_SRQ");
                Ok(RpcHandleStatus::Success)
            },
            Self::PROC_DEVICE_DOCMD => {
                log::debug!("PROC_DEVICE_DOCMD");
                // WTF does DOCMD even do?
                // It's a pain to pack/unpack and doesn't seem to be commonly used
                // so I'm not gonna bother with it.
                Ok(RpcHandleStatus::NoSuchProcedure)
            },
            Self::PROC_DESTROY_LINK => {
                log::debug!("PROC_DESTROY_LINK");
                Ok(RpcHandleStatus::Success)
            },
            Self::PROC_CREATE_INTR_CHAN => {
                log::debug!("PROC_CREATE_INTR_CHAN");
                Ok(RpcHandleStatus::Success)
            },
            Self::PROC_DESTROY_INTR_CHAN => {
                log::debug!("PROC_DESTROY_INTR_CHAN");
                Ok(RpcHandleStatus::Success)
            },
            _ => Ok(RpcHandleStatus::NoSuchProcedure)
        }
    }

    fn rpc_reply(&self, _reply: &mut XdrReader) -> Result<(), XdrError> {
        unimplemented!()
    }
}