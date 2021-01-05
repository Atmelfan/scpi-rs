//! # portmapper.rs
//!
//! Implements a limited version of port mapper (see https://tools.ietf.org/html/rfc1833) for
//! use with vxi-11 RPC protocol. Use this if the host is not capable of running a independent
//! version of port mapper/rpcbind.
//!
//!

use arrayvec::{ArrayVec, Array};
use crate::vxi11::xdr::{XdrEnum, XdrError, XdrUnpack, XdrReader, XdrPack, XdrWriter};
use crate::SocketPool;
use crate::vxi11::rpc::{RpcService, RpcHandler, RpcHandleStatus};
use smoltcp::socket::{SocketHandle, SocketSet};
use core::cell::RefCell;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Iproto {
    Tcp,
    Udp
}

impl XdrEnum for Iproto {
    fn xdr_to_discriminant(&self) -> i32 {
        match self {
            Iproto::Tcp => 6,
            Iproto::Udp => 17
        }
    }

    fn xdr_from_discriminant(d: i32) -> Result<Self, XdrError> {
        match d {
            6 => Ok(Self::Tcp),
            17 => Ok(Self::Udp),
            _ => Err(XdrError::InvalidDiscriminant)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Mapping {
    prog: u32,
    vers: u32,
    prot: Iproto,
    port: u32,
}

impl Mapping {

    pub fn new(prog: u32, vers: u32, prot: Iproto, port: u32) -> Self {
        Mapping {
            prog,
            vers,
            prot,
            port
        }
    }
}

impl Default for Mapping {
    fn default() -> Self {
        Mapping {
            prog: 0,
            vers: 0,
            prot: Iproto::Tcp,
            port: 0
        }
    }
}

impl XdrUnpack for Mapping {
    fn xdr_unpack(&mut self, reader: &mut XdrReader) -> Result<(), XdrError> {
        reader.read_u32(&mut self.prog)?;
        reader.read_u32(&mut self.vers)?;
        reader.read_enum(&mut self.prot)?;
        reader.read_u32(&mut self.port)
    }
}

impl XdrPack for Mapping {
    fn xdr_pack<A>(&self, writer: &mut XdrWriter<A>) -> Result<(), XdrError> where A: Array<Item=u8> {
        writer.write_u32(self.prog)?;
        writer.write_u32(self.vers)?;
        writer.write_enum(self.prot)?;
        writer.write_u32(self.port)
    }
}

pub struct PortMapper<M, A, T> where
    M: Array<Item = Mapping>,
    A: Array<Item = (SocketHandle, ArrayVec<T>)>,
    T: Array<Item = u8>  {
    mappings: RefCell<ArrayVec<M>>,
    rpc: RefCell<RpcService<A, T>>
}

impl<M, A, T> PortMapper<M, A, T> where
    M: Array<Item = Mapping>,
    A: Array<Item = (SocketHandle, ArrayVec<T>)>,
    T: Array<Item = u8>   {

    const PMAP_PROG: u32 = 100000;
    const PMAP_VERS: u32 = 2;
    const PMAP_PORT: u32 = 111;

    const PMAPPROC_NULL: u32 = 0;
    const PMAPPROC_SET: u32 = 1;
    const PMAPPROC_UNSET: u32 =2;
    const PMAPPROC_GETPORT: u32 = 3;
    const PMAPPROC_DUMP: u32 = 4;
    const PMAPPROC_CALLIT: u32 = 5;


    pub fn new(tcp: Option<SocketHandle>, udp: Option<SocketHandle>) -> Self {
        PortMapper {
            mappings: RefCell::new(ArrayVec::new()),
            rpc: RefCell::new(RpcService::new(
                Self::PMAP_PROG,
                Self::PMAP_VERS,
                Self::PMAP_PORT,
                tcp, udp))
        }
    }

    fn is_port_bound(&self, prog: u32, vers: u32, prot: Iproto) -> bool {
        for map in self.mappings.borrow().iter() {
            if map.prog == prog &&
                map.vers == vers &&
                map.prot == prot {
                return true;
            }
        }
        false
    }

    pub fn set(&self, prog: u32, vers: u32, prot: Iproto, port: u32) -> bool {
        if !self.is_port_bound(port, vers, prot) {
            self.mappings.borrow_mut().try_push(Mapping::new(prog, vers, prot, port)).is_ok()
        }else {
            false
        }
    }

    pub fn unset(&self, prog: u32, vers: u32) -> bool {
        self.mappings.borrow_mut().retain(|map| !(map.prog == prog && map.vers == vers));
        true
    }

    pub fn register_self(&self) -> bool {
        self.set(Self::PMAP_PROG, Self::PMAP_VERS, Iproto::Tcp, Self::PMAP_PORT) &&
            self.set(Self::PMAP_PROG, Self::PMAP_VERS, Iproto::Udp, Self::PMAP_PORT)
    }

    pub fn serve<X>(&mut self, sm: &mut SocketPool<X>, sockets: &mut SocketSet) -> smoltcp::Result<()>
        where X: Array<Item=SocketHandle> {
        self.rpc.borrow_mut().serve(sm, sockets, self)
    }
}

impl<M, A, T> RpcHandler<T> for PortMapper<M, A, T> where
    M: Array<Item = Mapping>,
    A: Array<Item = (SocketHandle, ArrayVec<T>)>,
    T: Array<Item = u8> {

    fn rpc_call(&self, proc: u32, _args: &mut XdrReader, _reply: &mut XdrWriter<T>) -> Result<RpcHandleStatus, XdrError> {

        match proc {
            /* void PMAPPROC_NULL(void) */
            Self::PMAPPROC_NULL => {
                log::info!("PMAPPROC_NULL(void)");
                Ok(RpcHandleStatus::Success)
            }
            /* bool PMAPPROC_SET(mapping) */
            Self::PMAPPROC_SET => {
                let mut mapping = Mapping::default();
                _args.read_structure(&mut mapping)?;
                log::info!("PMAPPROC_SET({:?})", mapping);
                _reply.write_bool(self.set(mapping.port,
                                           mapping.vers, mapping.prot, mapping.port))?;
                Ok(RpcHandleStatus::Success)
            }
            /* bool PMAPPROC_UNSET(mapping) */
            Self::PMAPPROC_UNSET => {
                let mut mapping = Mapping::default();
                _args.read_structure(&mut mapping)?;
                log::info!("PMAPPROC_UNSET({:?})", mapping);
                _reply.write_bool(self.unset(mapping.port, mapping.vers))?;
                Ok(RpcHandleStatus::Success)
            }
            /* unsigned int PMAPPROC_GETPORT(mapping) */
            Self::PMAPPROC_GETPORT => {
                let mut mapping = Mapping::default();
                _args.read_structure(&mut mapping)?;
                log::info!("PMAPPROC_GETPORT({:?})", mapping);

                let mut port = 0;
                for map in self.mappings.borrow().iter() {
                    if map.prog == mapping.prog &&
                        map.vers == mapping.vers &&
                        map.prot == mapping.prot {
                        port = map.port;
                    }
                }
                _reply.write_u32(port)?;
                Ok(RpcHandleStatus::Success)
            }
            /* pmaplist PMAPPROC_DUMP(void) */
            Self::PMAPPROC_DUMP => {
                log::info!("PMAPPROC_DUMP(void)");
                for map in self.mappings.borrow().iter() {
                    _reply.write_u32(1)?;
                    _reply.write_structure(map)?;
                }
                _reply.write_u32(0)?;
                Ok(RpcHandleStatus::Success)
            }
            /* call_result PMAPPROC_CALLIT(call_args) */
            Self::PMAPPROC_CALLIT => {
                log::info!("PMAPPROC_CALLIT(???)");
                // Not supported,
                // return false to trigger a error
                Ok(RpcHandleStatus::NoSuchProcedure)
            }
            _ => Ok(RpcHandleStatus::NoSuchProcedure)
        }
    }
}