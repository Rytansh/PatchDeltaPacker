use crate::build::patcher::structs::PatchEntry;

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
        patches: Vec<PatchEntry>,
        target: String,
    },
    PatchDownload {
        patch: PatchEntry,
        resume_offset: u64,
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
    OffsetMismatch,
    DownloadInfoNotFound,
    FatalError,
}
