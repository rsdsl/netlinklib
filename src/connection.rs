use crate::Result;

use rtnetlink::Handle;

#[derive(Debug)]
pub struct Connection(Handle);

impl Connection {
    /// Creates a new connection and handle to rtnetlink and spawns the connection task.
    /// Can be used to interact with rtnetlink by enabling certain crate features
    /// and calling the methods they provide.
    pub fn new() -> Result<Self> {
        let (conn, handle, _) = rtnetlink::new_connection()?;
        tokio::spawn(conn);

        Ok(Self(handle))
    }

    /// Returns a reference to the underlying [`rtnetlink::Handle`].
    pub(crate) fn handle(&self) -> &Handle {
        &self.0
    }
}
