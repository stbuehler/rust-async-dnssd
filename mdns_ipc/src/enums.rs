
#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,IpcStruct,IpcStructDisplay)]
pub struct IpcFlags(pub u32);
pub const IPC_FLAGS_NOREPLY: IpcFlags = IpcFlags(0x00000001);

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,IpcStruct,IpcStructDisplay)]
pub struct Operation(pub u32);

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,IpcStruct,IpcStructDisplay)]
pub struct Flags(pub u32);
pub const FLAGS_MORE_COMING              : Flags = Flags(0x00000001);
pub const FLAGS_AUTO_TRIGGER             : Flags = Flags(0x00000001);
pub const FLAGS_ADD                      : Flags = Flags(0x00000002);
pub const FLAGS_DEFAULT                  : Flags = Flags(0x00000004);
pub const FLAGS_NO_AUTO_RENAME           : Flags = Flags(0x00000008);
pub const FLAGS_SHARED                   : Flags = Flags(0x00000010);
pub const FLAGS_UNIQUE                   : Flags = Flags(0x00000020);
pub const FLAGS_BROWSE_DOMAINS           : Flags = Flags(0x00000040);
pub const FLAGS_REGISTRATION_DOMAINS     : Flags = Flags(0x00000080);
pub const FLAGS_LONG_LIVED_QUERY         : Flags = Flags(0x00000100);
pub const FLAGS_ALLOW_REMOTE_QUERY       : Flags = Flags(0x00000200);
pub const FLAGS_FORCE_MULTICAST          : Flags = Flags(0x00000400);
pub const FLAGS_KNOWN_UNIQUE             : Flags = Flags(0x00000800);
pub const FLAGS_RETURN_INTERMEDIATES     : Flags = Flags(0x00001000);
pub const FLAGS_NON_BROWSABLE            : Flags = Flags(0x00002000);
pub const FLAGS_SHARE_CONNECTION         : Flags = Flags(0x00004000);
pub const FLAGS_SUPPRESS_UNUSABLE        : Flags = Flags(0x00008000);
pub const FLAGS_TIMEOUT                  : Flags = Flags(0x00010000);
pub const FLAGS_INCLUDE_P2P              : Flags = Flags(0x00020000);
pub const FLAGS_WAKE_ON_RESOLVE          : Flags = Flags(0x00040000);
pub const FLAGS_BACKGROUND_TRAFFIC_CLASS : Flags = Flags(0x00080000);
pub const FLAGS_INCLUDE_AWDL             : Flags = Flags(0x00100000);
pub const FLAGS_VALIDATE                 : Flags = Flags(0x00200000);
pub const FLAGS_SECURE                   : Flags = Flags(0x00200010);
pub const FLAGS_INSECURE                 : Flags = Flags(0x00200020);
pub const FLAGS_BOGUS                    : Flags = Flags(0x00200040);
pub const FLAGS_INDETERMINATE            : Flags = Flags(0x00200080);
pub const FLAGS_UNICAST_RESPONSE         : Flags = Flags(0x00400000);
pub const FLAGS_VALIDATE_OPTIONAL        : Flags = Flags(0x00800000);
pub const FLAGS_WAKE_ONLY_SERVICE        : Flags = Flags(0x01000000);
pub const FLAGS_THRESHOLD_ONE            : Flags = Flags(0x02000000);
pub const FLAGS_THRESHOLD_REACHED        : Flags = Flags(0x02000000);
pub const FLAGS_THRESHOLD_FINDER         : Flags = Flags(0x04000000);
pub const FLAGS_DENY_CELLULAR            : Flags = Flags(0x08000000);
pub const FLAGS_SERVICE_INDEX            : Flags = Flags(0x10000000);
pub const FLAGS_DENY_EXPENSIVE           : Flags = Flags(0x20000000);
pub const FLAGS_PATH_EVALUATION_DONE     : Flags = Flags(0x40000000);

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,IpcStruct,IpcStructDisplay)]
pub struct ErrorCode(pub i32);
pub const ERROR_NO_ERROR                     : ErrorCode = ErrorCode(0);
pub const ERROR_UNKNOWN                      : ErrorCode = ErrorCode(-65537);
pub const ERROR_NO_SUCH_NAME                 : ErrorCode = ErrorCode(-65538);
pub const ERROR_NO_MEMORY                    : ErrorCode = ErrorCode(-65539);
pub const ERROR_BAD_PARAM                    : ErrorCode = ErrorCode(-65540);
pub const ERROR_BAD_REFERENCE                : ErrorCode = ErrorCode(-65541);
pub const ERROR_BAD_STATE                    : ErrorCode = ErrorCode(-65542);
pub const ERROR_BAD_FLAGS                    : ErrorCode = ErrorCode(-65543);
pub const ERROR_UNSUPPORTED                  : ErrorCode = ErrorCode(-65544);
pub const ERROR_NOT_INITIALIZED              : ErrorCode = ErrorCode(-65545);
pub const ERROR_ALREADY_REGISTERED           : ErrorCode = ErrorCode(-65547);
pub const ERROR_NAME_CONFLICT                : ErrorCode = ErrorCode(-65548);
pub const ERROR_INVALID                      : ErrorCode = ErrorCode(-65549);
pub const ERROR_FIREWALL                     : ErrorCode = ErrorCode(-65550);
pub const ERROR_INCOMPATIBLE                 : ErrorCode = ErrorCode(-65551);
pub const ERROR_BAD_INTERFACE_INDEX          : ErrorCode = ErrorCode(-65552);
pub const ERROR_REFUSED                      : ErrorCode = ErrorCode(-65553);
pub const ERROR_NO_SUCH_RECORD               : ErrorCode = ErrorCode(-65554);
pub const ERROR_NO_AUTH                      : ErrorCode = ErrorCode(-65555);
pub const ERROR_NO_SUCH_KEY                  : ErrorCode = ErrorCode(-65556);
pub const ERROR_NAT_TRAVERSAL                : ErrorCode = ErrorCode(-65557);
pub const ERROR_DOUBLE_NAT                   : ErrorCode = ErrorCode(-65558);
pub const ERROR_BAD_TIME                     : ErrorCode = ErrorCode(-65559);
pub const ERROR_BAD_SIG                      : ErrorCode = ErrorCode(-65560);
pub const ERROR_BAD_KEY                      : ErrorCode = ErrorCode(-65561);
pub const ERROR_TRANSIENT                    : ErrorCode = ErrorCode(-65562);
pub const ERROR_SERVICE_NOT_RUNNING          : ErrorCode = ErrorCode(-65563);
pub const ERROR_NAT_PORT_MAPPING_UNSUPPORTED : ErrorCode = ErrorCode(-65564);
pub const ERROR_NAT_PORT_MAPPING_DISABLED    : ErrorCode = ErrorCode(-65565);
pub const ERROR_NO_ROUTER                    : ErrorCode = ErrorCode(-65566);
pub const ERROR_POLLING_MODE                 : ErrorCode = ErrorCode(-65567);
pub const ERROR_TIMEOUT                      : ErrorCode = ErrorCode(-65568);

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,IpcStruct,IpcStructDisplay)]
pub struct InterfaceIndex(pub u32);
pub const INTERFACE_INDEX_ANY        : InterfaceIndex = InterfaceIndex(0);
pub const INTERFACE_INDEX_LOCAL_ONLY : InterfaceIndex = InterfaceIndex(!0);
pub const INTERFACE_INDEX_UNICAST    : InterfaceIndex = InterfaceIndex(!1);
pub const INTERFACE_INDEX_P2P        : InterfaceIndex = InterfaceIndex(!2);
pub const INTERFACE_INDEX_BLE        : InterfaceIndex = InterfaceIndex(!3);

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,IpcStruct,IpcStructDisplay)]
pub struct RRClass(pub u32);
pub const RR_CLASS_IN : RRClass = RRClass(1);

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,IpcStruct,IpcStructDisplay)]
pub struct RRType(pub u32);
pub const RR_TYPE_A    : RRType = RRType(1);
pub const RR_TYPE_PTR  : RRType = RRType(12);
pub const RR_TYPE_TXT  : RRType = RRType(16);
pub const RR_TYPE_AAAA : RRType = RRType(28);
pub const RR_TYPE_SRV  : RRType = RRType(33);
