use crossbeam_channel::{Receiver, Sender, TrySendError};
use anyhow::Result;
use super::{DisplayBuf, Renderer};

/// A renderer that feeds it display buffer back over a channel.
pub struct ChannelRenderer {
    displaybuf: DisplayBuf,
    sender: Sender<DisplayBuf>,
    receiver: Receiver<DisplayBuf>,
}

impl ChannelRenderer {
    pub fn get_receiver(&self) -> Receiver<DisplayBuf> {
        self.receiver.clone()
    }
}

impl Renderer for ChannelRenderer {
    /// Creates a new renderer with a screen of the given size
    fn new(width: u16, height: u16) -> Result<Self> {
        let (sender, receiver) = crossbeam_channel::bounded(1);
        Ok(Self {
            displaybuf: DisplayBuf::new(width, height),
            sender,
            receiver,
        })
    }

    fn buffer_mut(&mut self) -> &mut DisplayBuf {
        &mut self.displaybuf
    }

    /// Renders changes to screen
    fn update(&mut self) -> Result<()> {
        let new_buffer = self.displaybuf.new_from_this();
        let buffer = std::mem::replace(&mut self.displaybuf, new_buffer);

        match self.sender.try_send(buffer) {
            Err(TrySendError::Full(_)) => Ok(()),
            e => Ok(e?),
        }
    }
}