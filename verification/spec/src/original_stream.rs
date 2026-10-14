// Copyright © 2015 CCP ehf. MIT license, retained from original Destiny.
//! Thunkers.cpp's old-orientation binary stream on a little-endian ABI.
//! Each scalar has the original width and bit representation. Eight balls,
//! four descriptors of each kind per ball and 4096 bytes bound callers.
//! Truncated/unknown-mode inputs are outside this produced-stream scope;
//! Blue allocation, short native reads/writes, ABI and object identity remain.

use crate::{
    BoundedList, OriginalBallInit, OriginalMode, OriginalParkBall, OriginalParkError,
    OriginalParkProjection, Vec3,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OriginalStream {
    bytes: [u8; 4096],
    length: usize,
}
impl Default for OriginalStream {
    fn default() -> Self {
        Self {
            bytes: [0; 4096],
            length: 0,
        }
    }
}
impl OriginalStream {
    #[must_use]
    pub const fn len(&self) -> usize {
        self.length
    }
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }
    #[must_use]
    pub fn get(&self, index: usize) -> Option<u8> {
        if index < self.length {
            self.bytes.get(index).copied()
        } else {
            None
        }
    }
    pub fn write<const N: usize>(&mut self, data: [u8; N]) -> Result<(), OriginalParkError> {
        if N > 4096_usize.saturating_sub(self.length) {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        let mut i = 0;
        while i < N {
            let Some(slot) = self.bytes.get_mut(self.length) else {
                return Err(OriginalParkError::OutsideDeclaredScope);
            };
            let Some(byte) = data.get(i).copied() else {
                return Err(OriginalParkError::OutsideDeclaredScope);
            };
            *slot = byte;
            self.length += 1;
            i += 1;
        }
        Ok(())
    }
    fn vector(&mut self, value: Vec3) -> Result<(), OriginalParkError> {
        self.write(value.x.to_le_bytes())?;
        self.write(value.y.to_le_bytes())?;
        self.write(value.z.to_le_bytes())
    }
    pub fn equals_bytes(&self, bytes: &[u8]) -> bool {
        if self.length != bytes.len() {
            return false;
        }
        let mut i = 0;
        while i < self.length {
            if self.get(i) != bytes.get(i).copied() {
                return false;
            }
            i += 1;
        }
        true
    }
}

struct Reader<'a> {
    stream: &'a OriginalStream,
    position: usize,
}
impl Reader<'_> {
    fn read<const N: usize>(&mut self) -> Result<[u8; N], OriginalParkError> {
        if N > self.stream.length.saturating_sub(self.position) {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        let mut result = [0_u8; N];
        let mut i = 0;
        while i < N {
            let Some(slot) = result.get_mut(i) else {
                return Err(OriginalParkError::OutsideDeclaredScope);
            };
            let Some(byte) = self.stream.get(self.position) else {
                return Err(OriginalParkError::OutsideDeclaredScope);
            };
            *slot = byte;
            self.position += 1;
            i += 1;
        }
        Ok(result)
    }
    fn byte(&mut self) -> Result<u8, OriginalParkError> {
        Ok(u8::from_le_bytes(self.read()?))
    }
    fn float(&mut self) -> Result<f32, OriginalParkError> {
        Ok(f32::from_le_bytes(self.read()?))
    }
    fn double(&mut self) -> Result<f64, OriginalParkError> {
        Ok(f64::from_le_bytes(self.read()?))
    }
    fn id(&mut self) -> Result<i64, OriginalParkError> {
        Ok(i64::from_le_bytes(self.read()?))
    }
    fn integer(&mut self) -> Result<i32, OriginalParkError> {
        Ok(i32::from_le_bytes(self.read()?))
    }
    fn count(&mut self) -> Result<u16, OriginalParkError> {
        Ok(u16::from_le_bytes(self.read()?))
    }
    fn vector(&mut self) -> Result<Vec3, OriginalParkError> {
        Ok(Vec3 {
            x: self.double()?,
            y: self.double()?,
            z: self.double()?,
        })
    }
}

fn mode_code(mode: OriginalMode) -> u8 {
    match mode {
        OriginalMode::Goto => 0,
        OriginalMode::Follow => 1,
        OriginalMode::Stop => 2,
        OriginalMode::Warp => 3,
        OriginalMode::Orbit => 4,
        OriginalMode::Missile => 5,
        OriginalMode::Mushroom => 6,
        OriginalMode::Boid => 7,
        OriginalMode::Troll => 8,
        OriginalMode::Miniball => 9,
        OriginalMode::Field => 10,
        OriginalMode::Rigid => 11,
        OriginalMode::Formation => 12,
    }
}
fn decode_mode(mode: u8) -> Result<OriginalMode, OriginalParkError> {
    match mode {
        0 => Ok(OriginalMode::Goto),
        1 => Ok(OriginalMode::Follow),
        2 => Ok(OriginalMode::Stop),
        3 => Ok(OriginalMode::Warp),
        4 => Ok(OriginalMode::Orbit),
        5 => Ok(OriginalMode::Missile),
        6 => Ok(OriginalMode::Mushroom),
        8 => Ok(OriginalMode::Troll),
        10 => Ok(OriginalMode::Field),
        11 => Ok(OriginalMode::Rigid),
        12 => Ok(OriginalMode::Formation),
        _ => Err(OriginalParkError::OutsideDeclaredScope),
    }
}

fn write_ball(
    ball: OriginalParkBall,
    stream: &mut OriginalStream,
) -> Result<(), OriginalParkError> {
    if ball.is_moribund {
        return Ok(());
    }
    if matches!(ball.mode, OriginalMode::Boid | OriginalMode::Miniball) {
        return Err(OriginalParkError::OutsideDeclaredScope);
    }
    let flags = u8::from(ball.is_free)
        | (u8::from(ball.is_global) << 1)
        | (u8::from(ball.is_massive) << 2)
        | (u8::from(ball.is_interactive) << 3)
        | (u8::from(ball.is_space_junk) << 4)
        | (u8::from(!ball.mini_boxes.is_empty()) << 5)
        | (u8::from(!ball.mini_balls.is_empty()) << 6)
        | (u8::from(!ball.mini_capsules.is_empty()) << 7);
    stream.write(ball.id.to_le_bytes())?;
    stream.write([mode_code(ball.mode)])?;
    stream.write(ball.radius.to_le_bytes())?;
    stream.vector(ball.position)?;
    stream.write([flags])?;
    if ball.mode != OriginalMode::Rigid {
        stream.write(ball.mass.to_le_bytes())?;
        stream.write(ball.is_cloaked.to_le_bytes())?;
        stream.write(ball.harmonic.to_le_bytes())?;
        stream.write(ball.corporation_id.to_le_bytes())?;
        stream.write(ball.alliance_id.to_le_bytes())?;
    }
    if ball.is_free {
        stream.write(ball.max_velocity.to_le_bytes())?;
        stream.vector(ball.velocity)?;
        stream.write(ball.agility.to_le_bytes())?;
        stream.write(ball.speed_fraction.to_le_bytes())?;
    }
    stream.write(ball.formation_id.to_le_bytes())?;
    match ball.mode {
        OriginalMode::Follow
        | OriginalMode::Orbit
        | OriginalMode::Formation
        | OriginalMode::Missile => {
            stream.write(ball.follow_id.to_le_bytes())?;
            stream.write(ball.follow_range.to_le_bytes())?;
            if ball.mode == OriginalMode::Missile {
                stream.write(ball.owner_id.to_le_bytes())?;
            }
            if matches!(ball.mode, OriginalMode::Formation | OriginalMode::Missile) {
                stream.write((ball.effect_stamp as i32).to_le_bytes())?;
            }
            if ball.mode == OriginalMode::Missile {
                stream.vector(ball.goto)?;
            }
        }
        OriginalMode::Goto => stream.vector(ball.goto)?,
        OriginalMode::Warp => {
            stream.vector(ball.goto)?;
            stream.write((ball.effect_stamp as i32).to_le_bytes())?;
            stream.write(ball.last_collision.to_le_bytes())?;
            stream.write(ball.follow_id.to_le_bytes())?;
            stream.write(ball.owner_id.to_le_bytes())?;
        }
        OriginalMode::Mushroom => {
            stream.write(ball.follow_range.to_le_bytes())?;
            stream.write(ball.goto.x.to_le_bytes())?;
            stream.write((ball.effect_stamp as i32).to_le_bytes())?;
            stream.write(ball.owner_id.to_le_bytes())?;
        }
        OriginalMode::Troll => stream.write((ball.effect_stamp as i32).to_le_bytes())?,
        _ => {}
    }
    if !ball.mini_balls.is_empty() {
        stream.write((ball.mini_balls.len() as u16).to_le_bytes())?;
    }
    let mut i = 0;
    while i < ball.mini_balls.len() {
        let Some(child) = ball.mini_balls.get(i) else {
            return Err(OriginalParkError::OutsideDeclaredScope);
        };
        stream.vector(child.position)?;
        stream.write(child.radius.to_le_bytes())?;
        i += 1;
    }
    if !ball.mini_capsules.is_empty() {
        stream.write((ball.mini_capsules.len() as u16).to_le_bytes())?;
    }
    i = 0;
    while i < ball.mini_capsules.len() {
        let Some(child) = ball.mini_capsules.get(i) else {
            return Err(OriginalParkError::OutsideDeclaredScope);
        };
        stream.vector(child.a)?;
        stream.vector(child.b)?;
        stream.write(child.radius.to_le_bytes())?;
        i += 1;
    }
    if !ball.mini_boxes.is_empty() {
        stream.write((ball.mini_boxes.len() as u16).to_le_bytes())?;
    }
    i = 0;
    while i < ball.mini_boxes.len() {
        let Some(child) = ball.mini_boxes.get(i) else {
            return Err(OriginalParkError::OutsideDeclaredScope);
        };
        stream.vector(child.corner)?;
        stream.vector(child.x)?;
        stream.vector(child.y)?;
        stream.vector(child.z)?;
        i += 1;
    }
    Ok(())
}

impl OriginalParkProjection {
    /// Exact original [] branch: both Python writers emit this fixed-width
    /// header before their empty iteration. This source-backed specialization
    /// excludes no input from the original empty-list scenario, and does not
    /// abstract any executed ball/child operation. Other list sizes retain the
    /// complete writer below. Native stream dispatch/ABI remain obligations.
    pub fn write_original_empty_balls(
        &self,
        ids: &BoundedList<i64, 8>,
    ) -> Result<OriginalStream, OriginalParkError> {
        if !ids.is_empty() {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        original_packet_header(1, self.current_time)
    }
    pub fn write_original_balls(
        &self,
        ids: &BoundedList<i64, 8>,
    ) -> Result<OriginalStream, OriginalParkError> {
        if ids.is_empty() {
            return self.write_original_empty_balls(ids);
        }
        if self.dynamical_orientation {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        let mut stream = original_packet_header(1, self.current_time)?;
        let mut i = 0;
        while i < ids.len() {
            let Some(id) = ids.get(i) else {
                return Err(OriginalParkError::OutsideDeclaredScope);
            };
            if self.contains(id) {
                write_ball(self.retained(id), &mut stream)?;
            }
            i += 1;
        }
        Ok(stream)
    }
    pub fn write_original_full_state(
        &self,
        source_id: Option<i64>,
    ) -> Result<OriginalStream, OriginalParkError> {
        if self.dynamical_orientation {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        let mut stream = original_packet_header(0, self.current_time)?;
        let balls = self.live_ball_sequence()?;
        let mut i = 0;
        let source = source_id
            .filter(|id| *id != -1 && self.contains(*id))
            .map(|id| self.retained(id));
        while i < balls.len() {
            let Some(ball) = balls.get(i) else {
                return Err(OriginalParkError::OutsideDeclaredScope);
            };
            if ball.id >= -(1_i64 << 30)
                && (!self.is_master
                    || source.is_none_or(|s| {
                        (s.new_bubble == ball.new_bubble || ball.is_global)
                            && (ball.is_cloaked == 0 || ball.id == s.id)
                    }))
            {
                write_ball(ball, &mut stream)?;
            }
            i += 1;
        }
        Ok(stream)
    }
    /// Produced-stream scope of the native decoder, retaining partial mode 1
    /// (restore state without children) and mode 2 (append state with children).
    pub fn read_original_state(
        &mut self,
        stream: &OriginalStream,
        partial: i32,
    ) -> Result<(), OriginalParkError> {
        if self.dynamical_orientation || !(0..=2).contains(&partial) {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        if partial == 0 {
            self.reset_stream_registries();
        }
        let mut reader = Reader {
            stream,
            position: 0,
        };
        let mut packets = 0;
        while reader.position < stream.length {
            if packets >= 8 {
                return Err(OriginalParkError::OutsideDeclaredScope);
            }
            packets += 1;
            if reader.byte()? > 1 {
                return Err(OriginalParkError::ValueError);
            }
            self.current_time = i64::from(reader.integer()?);
            let mut additions = BoundedList::<i64, 8>::new();
            let mut count = 0;
            while reader.position < stream.length {
                if count >= 8 {
                    return Err(OriginalParkError::OutsideDeclaredScope);
                }
                count += 1;
                let id = reader.id()?;
                let mode = decode_mode(reader.byte()?)?;
                let radius = reader.float()?;
                let position = reader.vector()?;
                let flags = reader.byte()?;
                let mut mass = 1.0e34;
                let mut cloak = 0_i8;
                let mut harmonic = -1_i64;
                let mut corporation = -1_i32;
                let mut alliance = -1_i32;
                if mode != OriginalMode::Rigid {
                    mass = reader.double()?;
                    cloak = i8::from_le_bytes(reader.read()?);
                    harmonic = reader.id()?;
                    corporation = reader.integer()?;
                    alliance = reader.integer()?;
                }
                let free = flags & 1 != 0;
                let mut maximum = 0.0;
                let mut velocity = Vec3::ZERO;
                let mut agility = 1.0;
                let mut fraction = 0.0;
                if free {
                    maximum = reader.float()?;
                    velocity = reader.vector()?;
                    agility = reader.float()?;
                    fraction = reader.float()?;
                }
                let actual_id = self.add_ball(OriginalBallInit {
                    id,
                    mass,
                    radius,
                    max_velocity: maximum,
                    is_free: free,
                    is_global: flags & 2 != 0,
                    is_massive: flags & 4 != 0,
                    is_interactive: flags & 8 != 0,
                    is_space_junk: flags & 16 != 0,
                    position,
                    velocity,
                    agility,
                    speed_fraction: fraction,
                });
                if actual_id != id || self.scope_error {
                    return Err(OriginalParkError::OutsideDeclaredScope);
                }
                let formation = i8::from_le_bytes(reader.read()?);
                if partial != 1 && !free {
                    self.remove_stream_children(id)?;
                }
                let mut ball = self.retained(id);
                ball.formation_id = formation;
                ball.harmonic = harmonic;
                ball.corporation_id = corporation;
                ball.alliance_id = alliance;
                ball.mode = mode;
                ball.is_cloaked = cloak;
                match mode {
                    OriginalMode::Follow
                    | OriginalMode::Orbit
                    | OriginalMode::Formation
                    | OriginalMode::Missile => {
                        ball.follow_id = reader.id()?;
                        ball.follow_range = reader.float()?;
                        if mode == OriginalMode::Missile {
                            ball.owner_id = reader.id()?;
                        }
                        if matches!(mode, OriginalMode::Formation | OriginalMode::Missile) {
                            ball.effect_stamp = i64::from(reader.integer()?);
                        }
                        if mode == OriginalMode::Missile {
                            ball.goto = reader.vector()?;
                        }
                    }
                    OriginalMode::Goto => ball.goto = reader.vector()?,
                    OriginalMode::Warp => {
                        ball.goto = reader.vector()?;
                        ball.effect_stamp = i64::from(reader.integer()?);
                        ball.last_collision = reader.double()?;
                        ball.follow_id = reader.id()?;
                        ball.owner_id = reader.id()?;
                    }
                    OriginalMode::Mushroom => {
                        ball.follow_range = reader.float()?;
                        ball.goto.x = reader.double()?;
                        ball.effect_stamp = i64::from(reader.integer()?);
                        ball.owner_id = reader.id()?;
                    }
                    OriginalMode::Troll => ball.effect_stamp = i64::from(reader.integer()?),
                    _ => {}
                }
                self.write_attribute(id, |b| *b = ball);
                if flags & 64 != 0 {
                    let children = reader.count()?;
                    let mut i = 0;
                    while i < children {
                        let center = reader.vector()?;
                        let radius = reader.float()?;
                        if partial != 1 {
                            self.add_mini_ball(id, center, radius)?;
                        }
                        i += 1;
                    }
                }
                if flags & 128 != 0 {
                    let children = reader.count()?;
                    let mut i = 0;
                    while i < children {
                        let a = reader.vector()?;
                        let b = reader.vector()?;
                        let radius = reader.float()?;
                        if partial != 1 {
                            self.add_mini_capsule(id, a, b, radius)?;
                        }
                        i += 1;
                    }
                }
                if flags & 32 != 0 {
                    let children = reader.count()?;
                    let mut i = 0;
                    while i < children {
                        let corner = reader.vector()?;
                        let x = reader.vector()?;
                        let y = reader.vector()?;
                        let z = reader.vector()?;
                        if partial != 1 {
                            self.add_mini_box(id, corner, x, y, z)?;
                        }
                        i += 1;
                    }
                }
                if !additions.push(id) {
                    return Err(OriginalParkError::OutsideDeclaredScope);
                }
            }
            let mut i = 0;
            while i < additions.len() {
                let Some(id) = additions.get(i) else {
                    return Err(OriginalParkError::OutsideDeclaredScope);
                };
                let ball = self.retained(id);
                if matches!(
                    ball.mode,
                    OriginalMode::Follow
                        | OriginalMode::Missile
                        | OriginalMode::Orbit
                        | OriginalMode::Formation
                ) && !self.contains(ball.follow_id)
                {
                    self.write_attribute(id, |b| b.mode = OriginalMode::Goto);
                }
                i += 1;
            }
        }
        Ok(())
    }
}

fn original_packet_header(packet: u8, time: i64) -> Result<OriginalStream, OriginalParkError> {
    let mut stream = OriginalStream::default();
    stream.write([packet])?;
    stream.write((time as i32).to_le_bytes())?;
    Ok(stream)
}
