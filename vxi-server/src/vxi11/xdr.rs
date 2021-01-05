use arrayvec::{ArrayVec, Array};
use byteorder::{NetworkEndian, ByteOrder};
use core::clone::Clone;
use core::marker::Sized;
use core::result::Result;

pub trait XdrEnum where Self: Sized {
    fn xdr_to_discriminant(&self) -> i32;
    fn xdr_from_discriminant(d: i32) -> Result<Self, XdrError>;
}

#[derive(Debug, Copy, Clone)]
pub enum XdrError {
    Insufficient,
    InvalidDiscriminant
}

pub struct XdrWriter<A: Array> {
    buffer: ArrayVec<A>
}

impl<A: Array> XdrWriter<A> where A: Array{
    pub fn new() -> XdrWriter<A>  {
        XdrWriter {
            buffer: ArrayVec::new()
        }
    }

    pub fn get_slice(&self) -> &[A::Item] {
        self.buffer.as_slice()
    }
}

impl<A> XdrWriter<A> where A: Array<Item=u8>{
    pub fn begin(&mut self) {
        self.buffer.clear();
    }

    pub fn finish(&mut self) -> Result<&[u8], XdrError> {
        Ok(self.buffer.as_slice())
    }

    pub fn write_slice(&mut self, x: &[u8]) -> Result<(), XdrError>{
        self.buffer.try_extend_from_slice(x).map_err(|_| XdrError::Insufficient)
    }

    pub fn write_i32(&mut self, x: i32) -> Result<(), XdrError> {
        let mut slice = [0u8; 4];
        NetworkEndian::write_i32(&mut slice, x);
        self.write_slice(&slice)
    }

    pub fn write_u32(&mut self, x: u32) -> Result<(), XdrError> {
        let mut slice = [0u8; 4];
        NetworkEndian::write_u32(&mut slice, x);
        self.write_slice(&slice)
    }

    pub fn write_enum<ENUM>(&mut self, e: ENUM) -> Result<(), XdrError>
        where ENUM: XdrEnum {
        let mut slice = [0u8; 4];
        NetworkEndian::write_i32(&mut slice, e.xdr_to_discriminant());
        self.write_slice(&slice)
    }

    pub fn write_bool(&mut self, x: bool) -> Result<(), XdrError> {
        let mut slice = [0u8; 4];
        NetworkEndian::write_i32(&mut slice, if x {1} else {0});
        self.write_slice(&slice)
    }

    pub fn write_i64(&mut self, x: i64) -> Result<(), XdrError> {
        let mut slice = [0u8; 8];
        NetworkEndian::write_i64(&mut slice, x);
        self.write_slice(&slice)
    }

    pub fn write_u64(&mut self, x: u64) -> Result<(), XdrError> {
        let mut slice = [0u8; 8];
        NetworkEndian::write_u64(&mut slice, x);
        self.write_slice(&slice)
    }

    pub fn write_f32(&mut self, x: f32) -> Result<(), XdrError> {
        let mut slice = [0u8; 4];
        NetworkEndian::write_f32(&mut slice, x);
        self.write_slice(&slice)
    }

    pub fn write_f64(&mut self, x: f64) -> Result<(), XdrError> {
        let mut slice = [0u8; 8];
        NetworkEndian::write_f64(&mut slice, x);
        self.write_slice(&slice)
    }

    pub fn write_fixed_opaque(&mut self, x: &[u8]) -> Result<(), XdrError> {
        self.write_slice(x)?;
        let pad = (x.len() / 4) * 4 - x.len();
        for _ in 0..pad {
            self.buffer.try_push(0).map_err(|_| XdrError::Insufficient)?;
        }
        Ok(())
    }

    pub fn write_variable_opaque(&mut self, x: &[u8]) -> Result<(), XdrError> {
        self.write_u32(x.len() as u32)?;
        self.write_fixed_opaque(x)
    }

    pub fn write_string(&mut self, x: &[u8]) -> Result<(), XdrError> {
        self.write_variable_opaque(x)
    }

    pub fn write_structure<OBJ>(&mut self, obj: &OBJ) -> Result<(), XdrError>
        where OBJ: XdrPack {
        obj.xdr_pack(self)
    }

    pub fn write_fixed_array<OBJ>(&mut self, obj: &[OBJ]) -> Result<(), XdrError>
        where OBJ: XdrPack {
        for obj in obj.iter() {
            obj.xdr_pack(self)?;
        }
        Ok(())
    }

    pub fn write_variable_array<OBJ>(&mut self, obj: &[OBJ]) -> Result<(), XdrError>
        where OBJ: XdrPack {
        self.write_u32(obj.len() as u32)?;
        for obj in obj.iter() {
            obj.xdr_pack(self)?;
        }
        Ok(())
    }
}

pub trait XdrPack {
    fn xdr_pack<A>(&self, writer: &mut XdrWriter<A>) -> Result<(), XdrError> where A: Array<Item=u8> ;
}


pub struct XdrReader<'a> {
    pos: usize,
    buffer: &'a [u8]
}

impl<'a> XdrReader<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            pos: 0,
            buffer
        }
    }

    pub fn get_pos(&self) -> usize {
        self.pos
    }

    pub fn read_slice(&mut self, x: &mut [u8]) -> Result<(), XdrError>{
        if self.pos + x.len() <= self.buffer.len() {
            let opos = self.pos;
            x.copy_from_slice(&self.buffer[opos..opos+x.len()]);
            self.pos += x.len();
            Ok(())
        }else{
            Err(XdrError::Insufficient)
        }
    }

    pub fn read_i32(&mut self, x: &mut i32) -> Result<(), XdrError>{
        let mut slice = [0u8; 4];
        self.read_slice(&mut slice)?;
        *x = NetworkEndian::read_i32(&slice);
        Ok(())
    }

    pub fn read_u32(&mut self, x: &mut u32) -> Result<(), XdrError>{
        let mut slice = [0u8; 4];
        self.read_slice(&mut slice)?;
        *x = NetworkEndian::read_u32(&slice);
        Ok(())
    }

    pub fn read_enum<ENUM>(&mut self, x: &mut ENUM) -> Result<(), XdrError>
        where ENUM: XdrEnum {
        let mut discriminant= 0;
        self.read_i32(&mut discriminant)?;
        *x = <ENUM as XdrEnum>::xdr_from_discriminant(discriminant)?;
        Ok(())
    }

    pub fn read_bool(&mut self, x: &mut bool) -> Result<(), XdrError>{
        let mut discriminant= 0;
        self.read_i32(&mut discriminant)?;
        match discriminant {
            0 => {
                *x = false;
                Ok(())
            },
            1 => {
                *x = true;
                Ok(())
            },
            _ => Err(XdrError::InvalidDiscriminant)
        }
    }

    pub fn read_i64(&mut self, x: &mut i64) -> Result<(), XdrError>{
        let mut slice = [0u8; 8];
        self.read_slice(&mut slice)?;
        *x = NetworkEndian::read_i64(&slice);
        Ok(())
    }

    pub fn read_u64(&mut self, x: &mut u64) -> Result<(), XdrError>{
        let mut slice = [0u8; 8];
        self.read_slice(&mut slice)?;
        *x = NetworkEndian::read_u64(&slice);
        Ok(())
    }

    pub fn read_f32(&mut self, x: &mut f32) -> Result<(), XdrError>{
        let mut slice = [0u8; 4];
        self.read_slice(&mut slice)?;
        *x = NetworkEndian::read_f32(&slice);
        Ok(())
    }

    pub fn read_f64(&mut self, x: &mut f64) -> Result<(), XdrError>{
        let mut slice = [0u8; 8];
        self.read_slice(&mut slice)?;
        *x = NetworkEndian::read_f64(&slice);
        Ok(())
    }

    pub fn read_fixed_opaque(&mut self, x: &mut [u8]) -> Result<(), XdrError>{
        let pad = (x.len() / 4) * 4 - x.len();
        self.read_slice(x)?;
        // Skip padding
        self.pos += pad;
        Ok(())
    }

    pub fn read_variable_opaque<A>(&mut self, x: &mut ArrayVec<A>) -> Result<(), XdrError>
        where A: Array<Item=u8> {
        let mut len= 0;
        self.read_i32(&mut len)?;
        if self.pos + (len as usize) <= self.buffer.len() {
            let pad = (len / 4) * 4 - len;
            x.clear();
            x.try_extend_from_slice(&self.buffer[self.pos..self.pos+(len as usize)]).unwrap();
            self.pos += (len + pad) as usize;
            Ok(())
        }else{
            Err(XdrError::Insufficient)
        }
    }

    pub fn read_variable_opaque_slice(&mut self) -> Result<&[u8], XdrError> {
        let mut len= 0;
        self.read_i32(&mut len)?;
        if self.pos + (len as usize) <= self.buffer.len() {
            let pad = (len / 4) * 4 - len;
            let opos = self.pos;
            self.pos += (len + pad) as usize;
            Ok(&self.buffer[opos..opos+(len as usize)])
        }else{
            Err(XdrError::Insufficient)
        }
    }


    pub fn read_structure<OBJ>(&mut self, obj: &mut OBJ) -> Result<(), XdrError>
        where OBJ: XdrUnpack {
        obj.xdr_unpack(self)
    }

    pub fn read_fixed_array<OBJ>(&mut self, obj: &mut [OBJ]) -> Result<(), XdrError>
        where OBJ: XdrUnpack {
        for o in obj.iter_mut() {
            o.xdr_unpack(self)?;
        }
        Ok(())
    }

    //No variable length objects

}

pub trait XdrUnpack {
    fn xdr_unpack(&mut self, reader: &mut XdrReader) -> Result<(), XdrError>;
}


