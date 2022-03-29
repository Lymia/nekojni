use enumset::EnumSetType;

#[derive(EnumSetType, Debug)]
pub enum CFlags {
    Public = 0,
    Final = 4,
    Super = 5,
    Interface = 9,
    Abstract = 10,
    Synthetic = 12,
    Annotation = 13,
    Enum = 14,
}

#[derive(EnumSetType, Debug)]
pub enum FFlags {
    Public = 0,
    Private = 1,
    Protected = 2,
    Static = 3,
    Final = 4,
    Volatile = 6,
    Transient = 7,
    Synthetic = 12,
    Enum = 14,
}

#[derive(EnumSetType, Debug)]
pub enum MFlags {
    Public = 0,
    Private = 1,
    Protected = 2,
    Static = 3,
    Final = 4,
    Synchronized = 5,
    Bridge = 6,
    Varargs = 7,
    Native = 8,
    Abstract = 10,
    Strict = 11,
    Synthetic = 12,
    Enum = 14,
}
