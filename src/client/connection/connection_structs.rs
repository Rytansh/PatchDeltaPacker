#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Packet {
    Connection,
    ConnectionAck,

    VersionRequest {
        current: String,
    },
    VersionResponse {
        latest: String,
    },

    PatchRequest {
        from: String,
        to: String,
    },
    PatchResponse {
        name: String,
        size: u64,
        checksum: [u8; 32],
    },
    PatchChunk {
        bytes: Vec<u8>,
    },
    PatchComplete,

    Error {
        code: ErrorCode,
    },

    ConnectionComplete,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ErrorCode {
    PatchNotFound,
    InvalidVersion,
    ChecksumMismatch,
    PermissionDenied,
    FatalError,
}
