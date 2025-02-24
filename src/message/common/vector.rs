use crate::prelude::PropertyPerm;
use crate::message::common::PropertyState;
use crate::timestamp::INDITimestamp;

/// Common trait for all INDI vector types, following the INDI protocol DTD specification.
/// 
/// This trait defines the common attributes shared by all INDI vectors as specified in the protocol DTD.
/// The order and optionality of attributes follows the specification exactly.
/// 
/// # String Handling
/// 
/// This trait uses `&str` for all string-related methods rather than `String`. This is a deliberate design choice:
/// 
/// - Using `&str` in trait methods provides efficient read-only access to string data
/// - It allows the trait to work with both owned `String` and borrowed `&str` types
/// - It follows Rust's best practice of "accept strings, return string slices"
/// - It avoids unnecessary cloning when reading data
/// 
/// While the underlying vector types store owned `String` for XML serialization purposes,
/// the trait provides a consistent and efficient interface for reading these values.
/// 
/// ```dtd
/// <!ATTLIST defTextVector
///     device    %nameValue;    #REQUIRED  name of Device
///     name      %nameValue;    #REQUIRED  name of Property
///     label     %labelValue;   #IMPLIED   GUI label, use name by default
///     group     %groupTag;     #IMPLIED   Property group membership, blank by default
///     state     %propertyState #REQUIRED  current state of Property
///     perm      %propertyPerm; #REQUIRED  ostensible Client controlability
///     timeout   %numberValue;  #IMPLIED   worse-case time to affect, 0 default, N/A for ro
///     timestamp %timeValue;    #IMPLIED   moment when these data were valid
///     message   %textValue;    #IMPLIED   commentary
/// >
/// ```
pub trait INDIVector {
    /// Type of the elements in this vector
    type Element;

    /// Get the device name this vector belongs to (#REQUIRED)
    fn device(&self) -> &str;
    
    /// Get the vector name (#REQUIRED)
    fn name(&self) -> &str;
    
    /// Get the vector label if any (#IMPLIED - use name by default)
    fn label(&self) -> Option<&str>;
    
    /// Get the vector group if any (#IMPLIED - blank by default)
    fn group(&self) -> Option<&str>;
    
    /// Get the vector state (#REQUIRED)
    fn state(&self) -> PropertyState;
    
    /// Get the permission level (#REQUIRED)
    fn perm(&self) -> PropertyPerm;
    
    /// Get the timeout value if any (#IMPLIED - 0 default, N/A for ro)
    fn timeout(&self) -> Option<f64>;
    
    /// Get the vector timestamp if any (#IMPLIED)
    fn timestamp(&self) -> Option<&INDITimestamp>;
    
    /// Get the vector message if any (#IMPLIED)
    fn message(&self) -> Option<&str>;

    /// Get the vector elements
    fn elements(&self) -> &[Self::Element];
}
