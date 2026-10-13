// Copyright © 2023 CCP ehf. MIT license, retained from original Destiny.
//! Native park side effects of the source Actions methods. Stream arguments
//! use the actual original-backed byte codec and queue. Eight callbacks bound
//! the fixture's append-only on_ball_made_global closure. Python weak-reference
//! connections, Native/Blue dispatch and production refinement remain required.

use crate::{ActionValue,BoundedList,NetworkActions,OriginalParkError,OriginalParkProjection};

pub struct OriginalServerParkActions {
    pub queue:NetworkActions,
    pub made_global_connected:bool,
    pub made_global_callback_parameters:BoundedList<i64,8>,
}
impl Default for OriginalServerParkActions {
    fn default()->Self { Self { queue:NetworkActions::default(),made_global_connected:false,
        made_global_callback_parameters:BoundedList::new() } }
}
impl OriginalServerParkActions {
    pub fn add_ball_to_client_parks(&mut self,park:&OriginalParkProjection,id:i64)->Result<(),OriginalParkError> {
        let mut ids=BoundedList::new();
        if !ids.push(id) { return Err(OriginalParkError::OutsideDeclaredScope); }
        let stream=park.write_original_balls(&ids)?;
        self.queue.add_stream_to_system_history(id,stream).map_err(|_|OriginalParkError::OutsideDeclaredScope)
    }
    pub fn make_ball_local(&mut self,park:&mut OriginalParkProjection,id:i64)->Result<(),OriginalParkError> {
        park.set_global(id,false);
        if !park.contains(id) { return Ok(()); }
        self.queue.add_ball_not_global(id,park.retained(id).new_bubble).map_err(|_|OriginalParkError::OutsideDeclaredScope)
    }
    pub fn make_ball_global(&mut self,park:&mut OriginalParkProjection,id:i64)->Result<(),OriginalParkError> {
        park.set_global(id,true);self.add_ball_to_client_parks(park,id)?;
        if self.made_global_connected && !self.made_global_callback_parameters.push(id) { return Err(OriginalParkError::OutsideDeclaredScope); }Ok(())
    }
    pub fn remove_ball(&mut self,park:&mut OriginalParkProjection,id:i64)->Result<(),OriginalParkError> {
        if !park.contains(id) { return Err(OriginalParkError::OutsideDeclaredScope); } // source dictionary indexing must exist
        if park.retained(id).is_global {
            self.queue._notify_remove_global_ball(ActionValue::Integer(id)).map_err(|_|OriginalParkError::OutsideDeclaredScope)?;
        }
        park.remove_ball(id,0);
        if park.scope_error { Err(OriginalParkError::OutsideDeclaredScope) } else { Ok(()) }
    }
    pub fn clean_up_followers(&mut self,park:&OriginalParkProjection)->Result<(),OriginalParkError> {
        let remote=park.remote_followers()?;let mut i=0;
        while i<remote.len() {
            let Some(id)=remote.get(i) else { return Err(OriginalParkError::OutsideDeclaredScope); };
            self.queue.stop(ActionValue::Integer(id)).map_err(|_|OriginalParkError::OutsideDeclaredScope)?;i+=1;
        }Ok(())
    }
}
