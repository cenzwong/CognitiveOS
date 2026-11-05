use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident, $prefix:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct $name(pub String);

        impl $name {
            pub fn new_v7() -> Self {
                Self(format!("{}-{}", $prefix, Uuid::now_v7()))
            }

            pub fn from_string(s: impl Into<String>) -> Self {
                Self(s.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl FromStr for $name {
            type Err = std::convert::Infallible;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(s.to_string()))
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }
    };
}

define_id!(ProcessId, "proc");
define_id!(WorkspaceId, "ws");
define_id!(PrincipalId, "prn");
define_id!(EventId, "evt");
define_id!(SliceId, "slc");
define_id!(OperationId, "op");
define_id!(CheckpointId, "chk");
define_id!(ApprovalId, "appr");
define_id!(SessionId, "ses");
define_id!(ProgramId, "prg");
define_id!(ModelProfileId, "mdl");
