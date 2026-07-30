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
        resume_offset: u64,
    },
    PatchResponse {
        file: String,
        version: String,
        remaining_size: u64,
        checksum: [u8; 32],
    },
    PatchDownload,
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

pub struct Session {
    pub download: Option<PendingDownload>,
}

pub struct PendingDownload {
    pub patch_entry: PatchEntry,
    pub resume_offset: u64,
}
