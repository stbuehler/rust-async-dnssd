# mdnsResponder IPC protocol

## Address family and ports

Depending on the platform the mdnsResponder service either listens on
TCP `127.0.0.1:5354` (windows) or unix socket
`${DNSSD_UDS_PATH:-/var/run/mDNSResponder}`.

When a separate connection is required to return the status response, on
windows the client passed the port number it has bound on 127.0.0.1, and
on unix the client passes a unix socket path.  If the path is empty an
file descriptor is passed along with the last byte of the message.

## Generic message encoding

Integers are always encoded in network order.  Keep in mind that port
numbers are often already stored in netword order and must not be
converted again.

Strings are stored with a null-terminating byte.  Some strings have a
maximum length specified (which includes the null-terminating byte).  It
follows that strings cannot contain a null byte apart from the
terminating one.

## Enums and simple typedefs

### IPC flags

	type IpcFlags = u32;

Don't send asynchronous replies for this request:

	const IPC_FLAGS_NOREPLY: IpcFlags = 0x00000001;

### Operations

	type Operation = u32;

The value for the various operations are listed in the section
describing the operation itself.

### Operation flags

	type Flags = u32;

Most flags are only request flags or response flags, and are only valid
in certain circumstances.

	const FLAGS_MORE_COMING              : Flags = 0x00000001;
	const FLAGS_AUTO_TRIGGER             : Flags = 0x00000001;
	const FLAGS_ADD                      : Flags = 0x00000002;
	const FLAGS_DEFAULT                  : Flags = 0x00000004;
	const FLAGS_NO_AUTO_RENAME           : Flags = 0x00000008;
	const FLAGS_SHARED                   : Flags = 0x00000010;
	const FLAGS_UNIQUE                   : Flags = 0x00000020;
	const FLAGS_BROWSE_DOMAINS           : Flags = 0x00000040;
	const FLAGS_REGISTRATION_DOMAINS     : Flags = 0x00000080;
	const FLAGS_LONG_LIVED_QUERY         : Flags = 0x00000100;
	const FLAGS_ALLOW_REMOTE_QUERY       : Flags = 0x00000200;
	const FLAGS_FORCE_MULTICAST          : Flags = 0x00000400;
	const FLAGS_KNOWN_UNIQUE             : Flags = 0x00000800;
	const FLAGS_RETURN_INTERMEDIATES     : Flags = 0x00001000;
	const FLAGS_NON_BROWSABLE            : Flags = 0x00002000;
	const FLAGS_SHARE_CONNECTION         : Flags = 0x00004000;
	const FLAGS_SUPPRESS_UNUSABLE        : Flags = 0x00008000;
	const FLAGS_TIMEOUT                  : Flags = 0x00010000;
	const FLAGS_INCLUDE_P2P              : Flags = 0x00020000;
	const FLAGS_WAKE_ON_RESOLVE          : Flags = 0x00040000;
	const FLAGS_BACKGROUND_TRAFFIC_CLASS : Flags = 0x00080000;
	const FLAGS_INCLUDE_AWDL             : Flags = 0x00100000;
	const FLAGS_VALIDATE                 : Flags = 0x00200000;
	const FLAGS_SECURE                   : Flags = 0x00200010;
	const FLAGS_INSECURE                 : Flags = 0x00200020;
	const FLAGS_BOGUS                    : Flags = 0x00200040;
	const FLAGS_INDETERMINATE            : Flags = 0x00200080;
	const FLAGS_UNICAST_RESPONSE         : Flags = 0x00400000;
	const FLAGS_VALIDATE_OPTIONAL        : Flags = 0x00800000;
	const FLAGS_WAKE_ONLY_SERVICE        : Flags = 0x01000000;
	const FLAGS_THRESHOLD_ONE            : Flags = 0x02000000;
	const FLAGS_THRESHOLD_REACHED        : Flags = 0x02000000;
	const FLAGS_THRESHOLD_FINDER         : Flags = 0x04000000;
	const FLAGS_DENY_CELLULAR            : Flags = 0x08000000;
	const FLAGS_SERVICE_INDEX            : Flags = 0x10000000;
	const FLAGS_DENY_EXPENSIVE           : Flags = 0x20000000;
	const FLAGS_PATH_EVALUATION_DONE     : Flags = 0x40000000;

### Error codes

	type ErrorCode = u32;

	const Error_NoError                   : ErrorCode = 0;
	const Error_Unknown                   : ErrorCode = -65537;
	const Error_NoSuchName                : ErrorCode = -65538;
	const Error_NoMemory                  : ErrorCode = -65539;
	const Error_BadParam                  : ErrorCode = -65540;
	const Error_BadReference              : ErrorCode = -65541;
	const Error_BadState                  : ErrorCode = -65542;
	const Error_BadFlags                  : ErrorCode = -65543;
	const Error_Unsupported               : ErrorCode = -65544;
	const Error_NotInitialized            : ErrorCode = -65545;
	const Error_AlreadyRegistered         : ErrorCode = -65547;
	const Error_NameConflict              : ErrorCode = -65548;
	const Error_Invalid                   : ErrorCode = -65549;
	const Error_Firewall                  : ErrorCode = -65550;
	const Error_Incompatible              : ErrorCode = -65551;
	const Error_BadInterfaceIndex         : ErrorCode = -65552;
	const Error_Refused                   : ErrorCode = -65553;
	const Error_NoSuchRecord              : ErrorCode = -65554;
	const Error_NoAuth                    : ErrorCode = -65555;
	const Error_NoSuchKey                 : ErrorCode = -65556;
	const Error_NATTraversal              : ErrorCode = -65557;
	const Error_DoubleNAT                 : ErrorCode = -65558;
	const Error_BadTime                   : ErrorCode = -65559;
	const Error_BadSig                    : ErrorCode = -65560;
	const Error_BadKey                    : ErrorCode = -65561;
	const Error_Transient                 : ErrorCode = -65562;
	const Error_ServiceNotRunning         : ErrorCode = -65563;
	const Error_NATPortMappingUnsupported : ErrorCode = -65564;
	const Error_NATPortMappingDisabled    : ErrorCode = -65565;
	const Error_NoRouter                  : ErrorCode = -65566;
	const Error_PollingMode               : ErrorCode = -65567;
	const Error_Timeout                   : ErrorCode = -65568;

### Interface index

	type InterfaceIndex = u32;

Reserved values:

	const INTERFACE_INDEX_ANY        : InterfaceIndex = 0;
	const INTERFACE_INDEX_LOCAL_ONLY : InterfaceIndex = !0;
	const INTERFACE_INDEX_UNICAST    : InterfaceIndex = !1;
	const INTERFACE_INDEX_P2P        : InterfaceIndex = !2;
	const INTERFACE_INDEX_BLE        : InterfaceIndex = !3;

### DNS resource record class

See [DNS CLASSes](https://www.iana.org/assignments/dns-parameters/dns-
parameters.xhtml#dns-parameters-2).

	type RRClass = u16;
	const RR_CLASS_IN : RRClass = 1;

### DNS resource record type

See [Resource Record (RR) TYPEs](https://www.iana.org/assignments/dns-
parameters/dns-parameters.xhtml#dns-parameters-4).

	type RRType = u16;
	// some examples
	const RR_TYPE_A : RRType = 1;
	const RR_TYPE_PTR : RRType = 12;
	const RR_TYPE_TXT : RRType = 16;
	const RR_TYPE_AAAA : RRType = 28;
	const RR_TYPE_SRV : RRType = 33;


## Message Header

Apart from the status responses all messages are prefixed with the
message header.

	struct MessagerHeaderV1 {
		/// Always 1.  Incompatible with future versions.
		version: u32,
		/// Length of data following the header.  Should not exceed 70000.
		data_len: u32,
		/// IPC flags
		ipc_flags: IpcFlags,
		/// Operation identifier
		op: Operation,
		/// Replies repeat the value from requests.  A cancel_request
		/// operation uses this field to identify the request to be
		/// cancelled.
		client_context: u64,
		/// Identifies record (unique per connection); the client assigns
		/// the number when creating a record.
		///
		/// `!0` is a reserved value associated with the default TXT record
		/// of a service registration.
		///
		/// Requests not targeting resource records should use 0.
		reg_index: u32,
	}

## Status response

`cancel_request` and `send_bpf` requests don't return a status response.
All other requests return a status response ASAP.  Clients may block
until they receive the status response.

The status response always starts with this struct:

	struct Status {
		error: ErrorCode,
	}

If the error is zero (success) some requests send along additional data;
this are defined by a `struct StatusResponse` in the operations below.

If the connection is shared, the service cannot send the status response
on the same connection.  In this cases a second connection is opened.
The client specifies where it listens for this connection by prefixing
the request data with the following struct:

	#[cfg(windows)]
	struct ResponseChannel {
		/// on 127.0.0.1
		port: u16,
	}
	#[cfg(unix)]
	struct ResponseChannel {
		/// if empty string (length 1) a file descriptor is attached to the
		/// last byte of the message data.  Such file descriptor should be
		/// one half of a socketpair().
		path: string,
	}

## Async Response Header

If a operation below doesn't specify a Response struct it doesn't have
async responses.

Async responses have an additional header after the message header.

	struct AsyncResponseHeader {
		flags: Flags,
		if_index: InterfaceIndex,
		error: ErrorCode,
	}

## Resource record data

	struct RRData {
		/// The length of the next field in bytes
		u16 length;
		u8 data[length];
	}

## Operations

### Connection request

Create a connection to run other shared requests on.

	const connection_request : Operation = 1;
	struct Request { }

### Register record request

Register a DNS resource record (requires a shared connection created by
`connection_request` or `connection_delegate_reques`).

Needs a unique record identifier in the message header.

	const reg_record_request : Operation = 2;
	struct Request {
		flags: Flags, // must have either unique or shared set
		if_index: InterfaceIndex,
		fullname: string[256],
		type: RRType,
		class: RRClass,
		data: RRData,
		ttl: u32,
	}
	const reg_record_reply_op : Operation = 69;
	struct Response {
	}


### Remove record request

The record to be deleted is identified in the message header.

	const remove_record_request : Operation = 3;
	struct Request {
		flags: Flags,
	}

### Enumerate domains request

Enumerate browsable or registerable domains.


	const enumeration_request : Operation = 4;
	struct Request {
		flags: Flags,
		if_index: InterfaceIndex,
	}
	const enumeration_reply_op : Operation = 64;
	struct Response {
		domain: string,
	}

### Register service request

	const reg_service_request : Operation = 5;
	struct Request {
		flags: Flags,
		if_index: InterfaceIndex,
		name: string[256],
		regtype: string[MAX_ESCAPED_DOMAIN_NAME],
		domain: string[MAX_ESCAPED_DOMAIN_NAME],
		host: string[MAX_ESCAPED_DOMAIN_NAME],
		port: u16,
		txt: RRData,
	}
	const reg_service_reply_op : Operation = 65;
	struct Response {
		name: string[MAX_DOMAIN_LABEL+1],
		type: string[MAX_ESCAPED_DOMAIN_NAME],
		domain: string[MAX_ESCAPED_DOMAIN_NAME],
	}

### Browse services request

	const browse_request : Operation = 6;
	struct Request {
		flags: Flags,
		if_index: InterfaceIndex,
		regtype: string[MAX_ESCAPED_DOMAIN_NAME],
		domain: string[MAX_ESCAPED_DOMAIN_NAME],
	}
	const browse_reply_op : Operation = 66;
	struct Response {
		name: string[MAX_DOMAIN_LABEL+1],
		type: string[MAX_ESCAPED_DOMAIN_NAME],
		domain: string[MAX_ESCAPED_DOMAIN_NAME],
	}

### Resolve service request

	const resolve_request : Operation = 7;
	struct Request {
		flags: Flags,
		if_index: InterfaceIndex,
		name: string[256],
		regtype: string[MAX_ESCAPED_DOMAIN_NAME],
		domain: string[MAX_ESCAPED_DOMAIN_NAME],
	}
	const resolve_reply_op : Operation = 67;
	struct Response {
		fullname: string[MAX_ESCAPED_DOMAIN_NAME],
		target: string[MAX_ESCAPED_DOMAIN_NAME],
		port: u16,
		txt: RRData,
	}

### Query record request

	const query_request : Operation = 8;
	struct Request {
		flags: Flags,
		if_index: InterfaceIndex,
		name: string[256],
		type: RRType,
		class: RRClass,
	}
	const query_reply_op : Operation = 68;
	struct Response {
		name: string[MAX_ESCAPED_DOMAIN_NAME],
		type: RRType,
		class: RRClass,
		data: RRData,
		ttl: u32,
	}

### Reconfirm record request

	const reconfirm_record_request : Operation = 9;
	struct Request {
		flags: Flags,
		if_index: InterfaceIndex,
		fullname: string[256],
		type: RRType,
		class: RRClass,
		data: RRData,
	}

### Add record request

Add record to service registration (requires a shared connection created
by `reg_service_request`.

Needs a unique record identifier in the message header.

	const add_record_request : Operation = 10;
	struct Request {
		flags: Flags,
		type: RRType,
		data: RRData,
		ttl: u32,
	}

### Update record request.

The record to be updated is identified in the message header.

	const update_record_request : Operation = 11;
	struct Request {
		flags: Flags,
		data: RRData,
		ttl: u32,
	}

### Set domain request

	const setdomain_request : Operation = 12;
	struct Request {
		flags: Flags,
		domain: string[MAX_ESCAPED_DOMAIN_NAME],
	}

### Get property request

	const getproperty_request : Operation = 13;
	struct Request {
		property: string[256],
	}
	struct StatusResponse {
		length: u32, // length of data field in bytes
		data: PropertyData,
	}
	// unused; no async Response:
	const getproperty_reply_op : Operation = 70;

	#### Property "DaemonVersion"

		struct PropertyData {
			/// version of implementation looked at:
			///
			/// #define _DNS_SD_H 7655009
			version: u32,
		}

### Port mapping request

	const port_mapping_request : Operation = 14;
	struct Request {
		flags: Flags,
		if_index: InterfaceIndex,
		protocol: u32,
		internal_port: u16,
		external_port: u16,
		ttl: u32,
	}
	const port_mapping_reply_op : Operation = 71;
	struct Response {
		external_address: u8[4], // IPv4
		protocol: u8,
		internal_port: u16,
		external_port: u16,
		ttl: u32,
	}

### Address info request

	const addrinfo_request : Operation = 15;
	struct Request {
		flags: Flags,
		if_index: InterfaceIndex,
		protocol: u32,
		hostname: string[256],
	}
	const addrinfo_reply_op : Operation = 72;
	struct Response {
		name: string[MAX_ESCAPED_DOMAIN_NAME],
		type: RRType,
		class: RRClass,
		data: RRData,
		ttl: u32,
	}

### Send BPF request

Also sends along a file descriptor attached to the last byte of the
message.

Used on OSX in some helper to open `/dev/bpf$i` (for `$i` in
`[0..100]`) and send it to the service.

The interface doesn't seem to be protected...

There is no status response for this request.

	const send_bpf : Operation = 16;
	struct Request {
		flags: Flags,
	}

### Get pid request

	const getpid_request : Operation = 17;
	struct Request {
		/// source port of a connection to find pid for
		src_port: u16,
	}
	struct StatusResponse {
		pid: u32,
	}

### Release request

Release all PTRs to the given service name in some special list.

OS X only.

	const release_request : Operation = 18;
	struct Request {
		flags: Flags,
		name: string[256],
		regtype: string[MAX_ESCAPED_DOMAIN_NAME],
		domain: string[MAX_ESCAPED_DOMAIN_NAME],
	}

### Connection delegate request.

OS X only.

Similar to `connection` request.  Unclear what the pid is about.

	const connection_delegate_request : Operation = 19;
	struct Request {
		pid: u32, // acts only as bool
	}

### Cancel request.

Cancel a pending request (identified by `client_context` in the message
header).

Only used for requests in a shared connection, otherwise closing the
connection is good enough.

There is no status response for this request.

	const cancel_request : Operation = 63;
	struct Request { }
	struct StatusResponse { }


## Shared connections

???
Under certain circumstances the error-code response for a request is not
send on the same connection.

This is basically the case if the connection represents a resource which
gets released on connection termination or some async query.

In these cases the data starts with a u16 port (AF_INET) / null-
terminated unix socket path (AF_UNIX) to send the response to. Empty
unix socket path means a file-descriptor was sent with the last byte of
the message (unix socket fd passing).

The server handling looks very complicated, buf the client usage is:

Either create single connections for all requests (apart from those
that extend an existing request, like record manipulation) or use
connection_request to create a new connection and share that; in this
case async responses use a separate socket for the error-code response.


Async:
- connection_request
- connection_delegate_request
- resolve_request
- query_request


Not async:
- getproperty_request
- cancel_request



Don't share an existing connection (the server might allow it though):
- getproperty_request
- getpid_request
- setdomain_request
- connection_request
- connection_delegate_request
- reconfirm_record_request

Always share a connection, but dont't use separate socket for response:
- cancel_request

Always share a connection and use separate socket for response:
- reg_record_request
  requires underlying connection_request or connection_delegate_request
- add_record_request
  requires underlying reg_service_request
- update_record_request
- remove_record_request

Can share a connection but use separate socket for response:
- resolve_request
- query_request
- addrinfo_request
- browse_request
- reg_service_request
- enumeration_request
- port_mapping_request
