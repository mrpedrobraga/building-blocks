use glam::{Affine3A, UVec3};

#[derive(Debug)]
pub enum ServerMessage {
    Connection(ServerConnectionMessage),
    World(ServerWorldMessage),
    Scene(ServerWorldMessage),
}

#[derive(Debug)]
pub enum ServerConnectionMessage {
    /// Server accepted the connection attempt! Yay!
    Connect {},
    /// Server disconnected you for some reason.
    Disconnect { reason: String },
}

#[derive(Debug)]
pub enum ServerWorldMessage {
    /// Client was transported to a new (or their first) world!
    EnterWorld {
        /// The identifier for the world.
        /// this is helpful for the client to create caches.
        id: String,
    },
    /// Player left the current world.
    /// Note that this isn't required to be emitted when a client
    /// goes straight from one world to another.
    ///
    /// So, really, this represents the client kicking you to world select screen (if there is one).
    LeaveWorld,
}

#[derive(Debug)]
pub enum ServerSceneMessage {
    /// Client was transported to a new (or their first) scene within a world!
    EnterScene {
        /// The identifier of the scene.
        id: String,
    },
    /// Player left the current scene.
    /// Note that this isn't required to be emitted when a client
    /// goes straight from one scene to another.
    ///
    /// So, really, this represents the client kicking you to a loading screen.
    LeaveScene {},
    /// Message is about a block group in the current scene.
    BlockGroup(ServerBlockGroupMessage),
}

#[derive(Debug)]
pub enum ServerBlockGroupMessage {
    /// Message that declares the existence of a block group!
    Define {
        /// The id of the block group!
        id: String,
        size: UVec3,
        transform: Affine3A,
    },
    /// Declares the content of a block group.
    /// TODO: This _needs_ to be streamed...
    /// perhaps in the `Define` message you can also get a
    /// `impl Stream<Block>`
    Content {
        id: String,
        blocks: Vec<BlockGroupBlock>,
    },
}

#[derive(Debug)]
pub struct BlockGroupBlock {
    pub index_in_block_palette: u32,
}
