// Copyright © 2003 CCP ehf. Copyright (c) 2026 CCP Games. MIT license.
// License: verification/licenses/carbon-math.txt (same CCP Games MIT notice).
//! Pinned Blue 5.1.2 default-version-zero marshal for the single-action trees
//! constructed by ParkUpdateBatcher. The root preserves list vs tuple. These
//! trees contain no repeated tuple/list/byte object; scalar integers and
//! Unicode names bypass CHECKREF in the source, hence their shared map is zero.
//! The source default string table contains RemoveBalls at index 107 and does
//! not contain AddBallsToPark. Custom tables/version/checksum/shared/cyclic or
//! other Python objects are outside this concrete tree scope, with errors.
//! Python object/refcount/allocation and native correspondence remain required.

use crate::{BoundedList,OriginalParkError,OriginalStream};

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum OriginalMarshalledAction {
    AddBalls { stream:OriginalStream },
    RemoveBalls { ids:BoundedList<i64,8> },
}
#[derive(Debug,Clone,Copy,PartialEq)]
pub struct OriginalMarshalledEvent {
    pub timestamp:i64,
    pub root_is_list:bool,
    pub action_count:usize,
    pub parameter_count:usize,
    pub action:OriginalMarshalledAction,
}

fn length(output:&mut OriginalStream,value:usize)->Result<(),OriginalParkError> {
    if value<255 { output.write([value as u8]) }
    else if value<=i32::MAX as usize { output.write([255])?;output.write((value as i32).to_le_bytes()) }
    else { Err(OriginalParkError::OutsideDeclaredScope) }
}
fn integer(output:&mut OriginalStream,value:i64)->Result<(),OriginalParkError> {
    match value {
        -1=>output.write([7]),0=>output.write([8]),1=>output.write([9]),
        -128..=127=> { output.write([6])?;output.write((value as i8).to_le_bytes()) },
        -32768..=32767=> { output.write([5])?;output.write((value as i16).to_le_bytes()) },
        -2147483648..=2147483647=> { output.write([4])?;output.write((value as i32).to_le_bytes()) },
        _=>Err(OriginalParkError::OutsideDeclaredScope), // source WriteLong not used by these examples
    }
}
fn append(output:&mut OriginalStream,input:&OriginalStream)->Result<(),OriginalParkError> {
    let mut i=0;while i<input.len() {
        let Some(byte)=input.get(i) else { return Err(OriginalParkError::OutsideDeclaredScope); };
        output.write([byte])?;i+=1;
    }Ok(())
}

pub fn marshal_original_event(event:OriginalMarshalledEvent)->Result<OriginalStream,OriginalParkError> {
    if event.action_count!=1 || event.parameter_count!=1 { return Err(OriginalParkError::OutsideDeclaredScope); }
    let mut output=OriginalStream::default();output.write([126])?;output.write(0_i32.to_le_bytes())?;
    output.write([if event.root_is_list { 39 } else { 37 }])?; // LIST1 or TUPLE1
    output.write([44])?;integer(&mut output,event.timestamp)?; // (stamp, action)
    output.write([44])?; // (action_name, parameters)
    match event.action {
        OriginalMarshalledAction::AddBalls{stream}=> {
            output.write([18])?;length(&mut output,14)?; // Unicode UTF8 length
            for byte in b"AddBallsToPark" { output.write([*byte])?; }
            output.write([37])?;output.write([19])?; // one parameter: bytes
            length(&mut output,stream.len())?;append(&mut output,&stream)?;
        }
        OriginalMarshalledAction::RemoveBalls{ids}=> {
            output.write([17,107])?; // actual source default string-table entry
            output.write([37])?;
            if ids.is_empty() { output.write([38])?; }
            else {
                if ids.len()==1 { output.write([39])?; } else { output.write([21])?;length(&mut output,ids.len())?; }
                let mut i=0;while i<ids.len() {
                    let Some(id)=ids.get(i) else { return Err(OriginalParkError::OutsideDeclaredScope); };
                    integer(&mut output,id)?;i+=1;
                }
            }
        }
    }
    Ok(output)
}

struct Reader<'a>{bytes:&'a OriginalStream,position:usize}
impl Reader<'_> {
    fn byte(&mut self)->Result<u8,OriginalParkError> {
        let Some(byte)=self.bytes.get(self.position) else { return Err(OriginalParkError::OutsideDeclaredScope); };
        self.position+=1;Ok(byte)
    }
    fn fixed<const N:usize>(&mut self)->Result<[u8;N],OriginalParkError> {
        let mut bytes=[0;N];let mut i=0;while i<N {
            let value=self.byte()?;let Some(slot)=bytes.get_mut(i) else { return Err(OriginalParkError::OutsideDeclaredScope); };
            *slot=value;i+=1;
        }Ok(bytes)
    }
    fn length(&mut self)->Result<usize,OriginalParkError> {
        let small=self.byte()?;
        if small==255 { let n=i32::from_le_bytes(self.fixed()?);if n<0 { return Err(OriginalParkError::OutsideDeclaredScope); }Ok(n as usize) }
        else { Ok(usize::from(small)) }
    }
    fn integer(&mut self)->Result<i64,OriginalParkError> {
        match self.byte()? {
            7=>Ok(-1),8=>Ok(0),9=>Ok(1),6=>Ok(i64::from(i8::from_le_bytes(self.fixed()?))),
            5=>Ok(i64::from(i16::from_le_bytes(self.fixed()?))),4=>Ok(i64::from(i32::from_le_bytes(self.fixed()?))),
            _=>Err(OriginalParkError::OutsideDeclaredScope),
        }
    }
    fn tag(&mut self,expected:u8)->Result<(),OriginalParkError> {
        if self.byte()?==expected { Ok(()) } else { Err(OriginalParkError::OutsideDeclaredScope) }
    }
}
pub fn unmarshal_original_event(bytes:&OriginalStream)->Result<OriginalMarshalledEvent,OriginalParkError> {
    let mut reader=Reader{bytes,position:0};reader.tag(126)?;
    if i32::from_le_bytes(reader.fixed()?)!=0 { return Err(OriginalParkError::OutsideDeclaredScope); }
    let (root_is_list,action_count)=match reader.byte()? { 39=>(true,1),37=>(false,1),_=>return Err(OriginalParkError::OutsideDeclaredScope) };
    let parameter_count;
    reader.tag(44)?;let timestamp=reader.integer()?;reader.tag(44)?;
    let action=match reader.byte()? {
        18=> {
            if reader.length()?!=14 { return Err(OriginalParkError::OutsideDeclaredScope); }
            for byte in b"AddBallsToPark" { reader.tag(*byte)?; }
            parameter_count=match reader.byte()? { 37=>1,_=>return Err(OriginalParkError::OutsideDeclaredScope) };
            reader.tag(19)?;let length=reader.length()?;
            if length>4096 { return Err(OriginalParkError::OutsideDeclaredScope); }
            let mut stream=OriginalStream::default();let mut i=0;
            while i<length { stream.write([reader.byte()?])?;i+=1; }
            OriginalMarshalledAction::AddBalls{stream}
        }
        17=> {
            reader.tag(107)?;
            parameter_count=match reader.byte()? { 37=>1,_=>return Err(OriginalParkError::OutsideDeclaredScope) };
            let count=match reader.byte()? { 38=>0,39=>1,21=>reader.length()?,_=>return Err(OriginalParkError::OutsideDeclaredScope) };
            if count>8 { return Err(OriginalParkError::OutsideDeclaredScope); }
            let mut ids=BoundedList::new();let mut i=0;
            while i<count { if !ids.push(reader.integer()?) { return Err(OriginalParkError::OutsideDeclaredScope); }i+=1; }
            OriginalMarshalledAction::RemoveBalls{ids}
        }
        _=>return Err(OriginalParkError::OutsideDeclaredScope),
    };
    if reader.position!=bytes.len() { return Err(OriginalParkError::OutsideDeclaredScope); }
    Ok(OriginalMarshalledEvent{timestamp,root_is_list,action_count,parameter_count,action})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_stock_integer_string_table_and_tuple_root_bytes() {
        let mut ids=BoundedList::new();assert!(ids.push(1002));
        let encoded=marshal_original_event(OriginalMarshalledEvent{timestamp:1,root_is_list:false,
            action_count:1,parameter_count:1,action:OriginalMarshalledAction::RemoveBalls{ids}}).unwrap();
        // Header/shared-count=0, TUPLE1, TUPLE2, INT_1, TUPLE2,
        // STR_TABLE 107, TUPLE1, LIST1, INT16 1002 (0x03ea).
        assert!(encoded.equals_bytes(&[126,0,0,0,0,37,44,9,44,17,107,37,39,5,0xea,3]));
        let decoded=unmarshal_original_event(&encoded).unwrap();
        assert!(!decoded.root_is_list && decoded.action_count==1 && decoded.parameter_count==1 && decoded.timestamp==1);
        assert!(decoded.action==OriginalMarshalledAction::RemoveBalls{ids});
    }
    #[test]
    fn source_unicode_buffer_and_list_root_bytes() {
        let mut stream=OriginalStream::default();stream.write([1,0,0,0,0]).unwrap();
        let encoded=marshal_original_event(OriginalMarshalledEvent{timestamp:0,root_is_list:true,
            action_count:1,parameter_count:1,action:OriginalMarshalledAction::AddBalls{stream}}).unwrap();
        let expected=[126,0,0,0,0,39,44,8,44,18,14,b'A',b'd',b'd',b'B',b'a',b'l',b'l',b's',b'T',b'o',b'P',b'a',b'r',b'k',37,19,5,1,0,0,0,0];
        assert!(encoded.equals_bytes(&expected));
        let decoded=unmarshal_original_event(&encoded).unwrap();
        assert!(decoded.root_is_list && decoded.action_count==1 && decoded.parameter_count==1 && decoded.timestamp==0);
        assert!(decoded.action==OriginalMarshalledAction::AddBalls{stream});
    }
}
