// Copyright © 2023 CCP ehf. MIT license, retained from original test fixtures.
//! Ordered defaultdict/list and character/client mapping from server helpers.
//! Eight entries bound these concrete fixtures. Output set views use membership;
//! Python hashing, aliases, weak references and native dispatch remain external.

use crate::BoundedList;

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum InterestScopeError { CapacityExceeded }

#[derive(Default)]
pub struct OriginalNetworkInterests {
    ball_characters:BoundedList<(i64,i64),8>,
    character_clients:BoundedList<(i64,Option<i64>),8>,
    ball_clients:BoundedList<(i64,i64),8>,
    ball_info_characters:BoundedList<(i64,i64),8>,
}
impl OriginalNetworkInterests {
    pub fn set_ball_info_characters(&mut self,ball:i64,characters:&[i64])->Result<(),InterestScopeError> {
        let mut retained=BoundedList::new();let mut i=0;
        while i<self.ball_info_characters.len() {
            if let Some((key,value))=self.ball_info_characters.get(i) {
                if key!=ball && !retained.push((key,value)) { return Err(InterestScopeError::CapacityExceeded); }
            }i+=1;
        }
        i=0;while i<characters.len() {
            let Some(character)=characters.get(i).copied() else { return Err(InterestScopeError::CapacityExceeded); };
            if !retained.push((ball,character)) { return Err(InterestScopeError::CapacityExceeded); }i+=1;
        }
        self.ball_info_characters=retained;Ok(())
    }
    pub fn ball_info_characters(&self,ball:i64)->Result<BoundedList<i64,8>,InterestScopeError> {
        matching(&self.ball_info_characters,ball)
    }
    pub fn add_character_interest(&mut self,ball:i64,character:i64)->Result<(),InterestScopeError> {
        if self.ball_characters.push((ball,character)) { Ok(()) } else { Err(InterestScopeError::CapacityExceeded) }
    }
    pub fn add_client_interest(&mut self,ball:i64,client:i64)->Result<(),InterestScopeError> {
        if self.ball_clients.push((ball,client)) { Ok(()) } else { Err(InterestScopeError::CapacityExceeded) }
    }
    pub fn map_to_client(&mut self,character:i64,client:Option<i64>)->Result<(),InterestScopeError> {
        let mut i=0;while i<self.character_clients.len() {
            if self.character_clients.get(i).is_some_and(|(key,_)|key==character) {
                return if self.character_clients.set(i,(character,client)) { Ok(()) } else { Err(InterestScopeError::CapacityExceeded) };
            }
            i+=1;
        }
        if self.character_clients.push((character,client)) { Ok(()) } else { Err(InterestScopeError::CapacityExceeded) }
    }
    #[must_use] pub fn client_for_character(&self,character:i64)->Option<i64> {
        let mut i=0;while i<self.character_clients.len() {
            if let Some((key,value))=self.character_clients.get(i) { if key==character { return value; } }i+=1;
        }None
    }
    pub fn characters_for_ball(&self,ball:i64)->Result<BoundedList<i64,8>,InterestScopeError> {
        matching(&self.ball_characters,ball)
    }
    pub fn clients_for_ball(&self,ball:i64)->Result<BoundedList<i64,8>,InterestScopeError> {
        matching(&self.ball_clients,ball)
    }
}
fn matching(entries:&BoundedList<(i64,i64),8>,key:i64)->Result<BoundedList<i64,8>,InterestScopeError> {
    let mut result=BoundedList::new();let mut i=0;
    while i<entries.len() {
        if let Some((candidate,value))=entries.get(i) { if candidate==key && !result.push(value) { return Err(InterestScopeError::CapacityExceeded); } }i+=1;
    }Ok(result)
}
