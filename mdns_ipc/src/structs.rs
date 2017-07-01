use mdns_ipc_core::{Const};

use enums::*;

const VERSION1: u32 = 1;
#[derive(IpcStruct,IpcStructDisplay)]
pub struct MessagerHeaderV1 {
	#[Const(VERSION1)]
	/// Always 1.  Incompatible with future versions.
	pub _version: Const,
	/// Length of data following the header.  Should not exceed 70000.
	pub data_len: u32,
	/// IPC flags
	pub ipc_flags: IpcFlags,
	/// Operation identifier
	pub op: Operation,
	/// Replies repeat the value from requests.  A cancel_request
	/// operation uses this field to identify the request to be
	/// cancelled.
	pub client_context: u64,
	/// Identifies record (unique per connection); the client assigns
	/// the number when creating a record.
	///
	/// `!0` is a reserved value associated with the default TXT record
	/// of a service registration.
	///
	/// Requests not targeting resource records should use 0.
	pub reg_index: u32,
}
