pub mod channel;

use anyhow::Result;

use std::ops::{Deref, DerefMut};
pub struct DisplayBuf {
    pub width: u16,
    pub height: u16,
    frame: Vec<u8>,
}

impl DisplayBuf {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            frame: Self::alloc_buf(width, height),
        }
    }

    pub fn new_from_this(&self) -> Self {
        Self {
            width: self.width,
            height: self.height,
            frame: Self::alloc_buf(self.width, self.height),
        }
    }

    fn alloc_buf(width: u16, height: u16) -> Vec<u8> {
        let buf_sz: usize = (width * height).into(); // GB pixels have 4 possible colours so only need 1 u8
        vec![0; buf_sz]
    }
}

impl Deref for DisplayBuf {
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.frame
    }
}

impl DerefMut for DisplayBuf {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.frame
    }
}

pub trait Renderer {
    /// Creates a new renderer with a screen of the given size
    fn new(width: u16, height: u16) -> Result<Self>
    where
        Self: Renderer + Sized;

    /// Renders changes to screen
    fn update(&mut self) -> Result<()>;

    /// Gets a reference to the back buffer
    fn buffer_mut(&mut self) -> &mut DisplayBuf;
}

pub struct NullRenderer {
    buffer: DisplayBuf,
}

impl Renderer for NullRenderer {
    fn new(width: u16, height: u16) -> Result<Self> {
        Ok(Self {
            buffer: DisplayBuf::new(width, height),
        })
    }

    fn update(&mut self) -> Result<()> {
        Ok(())
    }

    #[inline(always)]
    fn buffer_mut(&mut self) -> &mut DisplayBuf {
        &mut self.buffer
    }
}
