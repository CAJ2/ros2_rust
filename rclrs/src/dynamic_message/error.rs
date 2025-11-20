use std::{error::Error, fmt};

/// An error related to creating a dynamic message based on the name of the message's type.
// TODO(luca) we need PartialEq (and maybe Eq?) for testing
#[derive(Debug)]
pub enum DynamicMessageError {
    /// The type support library was not found because no matching prefix was sourced.
    RequiredPrefixNotSourced {
        /// The package that was not found.
        package: String,
    },
    /// The message type does not have the shape `<package>/msg/<msg_name>`.
    InvalidMessageTypeSyntax {
        /// The message type passed to rclrs.
        input: String,
    },
    /// The message type could not be found in the package.
    InvalidMessageType,
    /// The operation expected a dynamic message of a different type.
    MessageTypeMismatch,
    /// Loading the type support library failed.
    LibraryLoadingError(libloading::Error),
    #[cfg(feature = "serde")]
    /// An error related to serializing or deserializing a dynamic message.
    SerdeMessage(String),
    #[cfg(feature = "serde")]
    /// A type is not supported for serde serialization/deserialization.
    TypeNotSupported(String),
    #[cfg(feature = "serde")]
    /// The UTF-8 encoding is invalid.
    InvalidUtf8Encoding,
}

impl fmt::Display for DynamicMessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequiredPrefixNotSourced { package } => {
                write!(f, "Package '{}' was not found in any prefix", package)
            }
            Self::InvalidMessageTypeSyntax { input } => write!(
                f,
                "The message type '{}' does not have the form <package>/msg/<msg_name>",
                input
            ),
            Self::InvalidMessageType => write!(f, "The message type was not found in the package"),
            Self::MessageTypeMismatch => write!(
                f,
                "The operation expected a dynamic message of a different type"
            ),
            Self::LibraryLoadingError(_) => write!(f, "Loading the type support library failed"),
            #[cfg(feature = "serde")]
            Self::SerdeMessage(msg) => write!(f, "Dynamic message serde error: {}", msg),
            #[cfg(feature = "serde")]
            Self::TypeNotSupported(ty) => write!(f, "Type not supported for serde: {}", ty),
            #[cfg(feature = "serde")]
            Self::InvalidUtf8Encoding => write!(f, "The UTF-8 encoding is invalid"),
        }
    }
}

impl Error for DynamicMessageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DynamicMessageError::LibraryLoadingError(lle) => Some(lle).map(|e| e as &dyn Error),
            _ => None,
        }
    }
}

impl PartialEq for DynamicMessageError {
    fn eq(&self, other: &Self) -> bool {
        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return false;
        }
        // TODO(luca) this is not very efficient, revisit
        return self.to_string() == other.to_string();
    }
}

impl Eq for DynamicMessageError {}
