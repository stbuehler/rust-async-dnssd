/// DNS CLASS
///
/// Originally QCLASS was a superset of CLASS; RFC 6895 now defines:
///
/// > There are currently two subcategories of DNS CLASSes: normal,
/// > data-containing classes; and QCLASSes that are only meaningful in
/// > queries or updates.
///
/// ## `ANY`
///
/// QTYPE 255 either (rules from RFC 6895):
///
/// - doesn't have a mnemonic, violating the existence rule
/// - has "*" as mnemonic, violating the formatting rule
/// - has "ANY" as mnemonic, violating the uniquess rule (class ANY)
///
/// The QCLASS `ANY` is mostly useless anyway and shouldn't be used in
/// normal queries.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Class(pub u16);

impl Class {
	/// CLASS Internet
	pub const IN: Class = Class(0x0001); // RFC 1035
	// CS = 0x0002, // "CSNET" (not just obsolete; unassigned in the IANA registry)
	/// CLASS "Chaos"
	pub const CH: Class = Class(0x0003); // "Chaos"
	/// CLASS "Hesiod"
	pub const HS: Class = Class(0x0004); // "Hesiod"
	/// QCLASS NONE
	pub const NONE: Class = Class(0x00fe); // RFC 2136
	/// QCLASS "*" (ANY)
	pub const ANY: Class = Class(0x00ff); // RFC 1035
}

/// DNS (RR)TYPE
///
/// Originally QTYPE was a superset of TYPE; RFC 6895 now defines:
///
/// > There are three subcategories of RRTYPE numbers: data TYPEs,
/// > QTYPEs, and Meta-TYPEs.
///
/// ## `ANY`
///
/// QTYPE 255 ("*") doesn't seem to have an official mnemonic; `ANY` is
/// used in most tools though.
///
/// The `ANY` mnemonic conflicts with the QCLASS `ANY` though...
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Type(pub u16);

impl Type {
	/// a host address
	pub const A: Type = Type(0x0001); // RFC 1035
	/// an authoritative name server
	pub const NS: Type = Type(0x0002); // RFC 1035
	/// a mail destination (OBSOLETE - use MX)
	pub const MD: Type = Type(0x0003); // RFC 1035
	/// a mail forwarder (OBSOLETE - use MX)
	pub const MF: Type = Type(0x0004); // RFC 1035
	/// the canonical name for an alias
	pub const CNAME: Type = Type(0x0005); // RFC 1035
	/// marks the start of a zone of authority
	pub const SOA: Type = Type(0x0006); // RFC 1035
	/// a mailbox domain name (EXPERIMENTAL)
	pub const MB: Type = Type(0x0007); // RFC 1035
	/// a mail group member (EXPERIMENTAL)
	pub const MG: Type = Type(0x0008); // RFC 1035
	/// a mail rename domain name (EXPERIMENTAL)
	pub const MR: Type = Type(0x0009); // RFC 1035
	/// a null RR (EXPERIMENTAL)
	pub const NULL: Type = Type(0x000a); // RFC 1035
	/// a well known service description
	pub const WKS: Type = Type(0x000b); // RFC 1035
	/// a domain name pointer
	pub const PTR: Type = Type(0x000c); // RFC 1035
	/// host information
	pub const HINFO: Type = Type(0x000d); // RFC 1035
	/// mailbox or mail list information
	pub const MINFO: Type = Type(0x000e); // RFC 1035
	/// mail exchange
	pub const MX: Type = Type(0x000f); // RFC 1035
	/// text strings
	pub const TXT: Type = Type(0x0010); // RFC 1035
	/// for Responsible Person
	pub const RP: Type = Type(0x0011); // RFC 1183
	/// for AFS Data Base location
	pub const AFSDB: Type = Type(0x0012); // RFC 1183
	/// for X.25 PSDN address
	pub const X25: Type = Type(0x0013); // RFC 1183
	/// for ISDN address
	pub const ISDN: Type = Type(0x0014); // RFC 1183
	/// for Route Through
	pub const RT: Type = Type(0x0015); // RFC 1183
	/// for NSAP address, NSAP style A record
	pub const NSAP: Type = Type(0x0016); // RFC 1706
	/// for domain name pointer, NSAP style
	pub const NSAP_PTR: Type = Type(0x0017); // RFC 1348
	/// for security signature
	pub const SIG: Type = Type(0x0018); // RFC 2535
	/// for security key
	pub const KEY: Type = Type(0x0019); // RFC 2535
	/// X.400 mail mapping information
	pub const PX: Type = Type(0x001a); // RFC 2163
	/// Geographical Position
	pub const GPOS: Type = Type(0x001b); // RFC 1712
	/// IP6 Address
	pub const AAAA: Type = Type(0x001c); // RFC 3596
	/// Location Information
	pub const LOC: Type = Type(0x001d); // RFC 1876
	/// Next Domain (OBSOLETE)
	pub const NXT: Type = Type(0x001e); // RFC 2535
	/// Endpoint Identifier
	pub const EID: Type = Type(0x001f); // Michael Patton: http://ana-3.lcs.mit.edu/~jnc/nimrod/dns.txt
	/// Nimrod Locator
	pub const NIMLOC: Type = Type(0x0020); // Michael Patton: http://ana-3.lcs.mit.edu/~jnc/nimrod/dns.txt
	/// Server Selection
	pub const SRV: Type = Type(0x0021); // RFC 2782
	/// ATM Address
	pub const ATMA: Type = Type(0x0022); // http://www.broadband-forum.org/ftp/pub/approved-specs/af-dans-0152.000.pdf
	/// Naming Authority Pointer
	pub const NAPTR: Type = Type(0x0023); // RFC 2168
	/// Key Exchanger
	pub const KX: Type = Type(0x0024); // RFC 2230
	/// CERT
	pub const CERT: Type = Type(0x0025); // RFC 4398
	/// A6 (OBSOLETE - use AAAA)
	pub const A6: Type = Type(0x0026); // RFC 2874
	/// DNAME
	pub const DNAME: Type = Type(0x0027); // RFC 6672
	/// SINK
	pub const SINK: Type = Type(0x0028); // Donald E Eastlake: http://tools.ietf.org/html/draft-eastlake-kitchen-sink
	/// OPT
	pub const OPT: Type = Type(0x0029); // RFC 6891
	/// APL
	pub const APL: Type = Type(0x002a); // RFC 3123
	/// Delegation Signer
	pub const DS: Type = Type(0x002b); // RFC 3658
	/// SSH Key Fingerprint
	pub const SSHFP: Type = Type(0x002c); // RFC 4255
	/// IPSECKEY
	pub const IPSECKEY: Type = Type(0x002d); // RFC 4025
	/// RRSIG
	pub const RRSIG: Type = Type(0x002e); // RFC 4034
	/// NSEC
	pub const NSEC: Type = Type(0x002f); // RFC 4034
	/// DNSKEY
	pub const DNSKEY: Type = Type(0x0030); // RFC 4034
	/// DHCID
	pub const DHCID: Type = Type(0x0031); // RFC 4701
	/// NSEC3
	pub const NSEC3: Type = Type(0x0032); // RFC 5155
	/// NSEC3PARAM
	pub const NSEC3PARAM: Type = Type(0x0033); // RFC 5155
	/// TLSA
	pub const TLSA: Type = Type(0x0034); // RFC 6698
	/// S/MIME cert association
	pub const SMIMEA: Type = Type(0x0035); // RFC 8162
	/// Host Identity Protocol
	pub const HIP: Type = Type(0x0037); // RFC 8005
	/// NINFO
	pub const NINFO: Type = Type(0x0038); // Jim Reid: https://tools.ietf.org/html/draft-reid-dnsext-zs-01
	/// RKEY
	pub const RKEY: Type = Type(0x0039); // Jim Reid: https://tools.ietf.org/html/draft-reid-dnsext-rkey-00
	/// Trust Anchor LINK
	pub const TALINK: Type = Type(0x003a); // Wouter Wijngaards
	/// Child DS
	pub const CDS: Type = Type(0x003b); // RFC 7344
	/// DNSKEY(s) the Child wants reflected in DS
	pub const CDNSKEY: Type = Type(0x003c); // RFC 7344
	/// OpenPGP Key
	pub const OPENPGPKEY: Type = Type(0x003d); // RFC 7929
	/// Child-To-Parent Synchronization
	pub const CSYNC: Type = Type(0x003e); // RFC 7477
	/// SPF
	pub const SPF: Type = Type(0x0063); // RFC 7208
	/// UINFO
	pub const UINFO: Type = Type(0x0064); // IANA-Reserved
	/// UID
	pub const UID: Type = Type(0x0065); // IANA-Reserved
	/// GID
	pub const GID: Type = Type(0x0066); // IANA-Reserved
	/// UNSPEC
	pub const UNSPEC: Type = Type(0x0067); // IANA-Reserved
	/// NID
	pub const NID: Type = Type(0x0068); // RFC 6742
	/// L32
	pub const L32: Type = Type(0x0069); // RFC 6742
	/// L64
	pub const L64: Type = Type(0x006a); // RFC 6742
	/// LP
	pub const LP: Type = Type(0x006b); // RFC 6742
	/// an EUI-48 address
	pub const EUI48: Type = Type(0x006c); // RFC 7043
	/// an EUI-64 address
	pub const EUI64: Type = Type(0x006d); // RFC 7043

	// 0x0080..0x00ff: meta and qtypes
	/// Transaction Key
	pub const TKEY: Type = Type(0x00f9); // RFC 2930
	/// Transaction Signature
	pub const TSIG: Type = Type(0x00fa); // RFC 2845
	/// incremental transfer
	pub const IXFR: Type = Type(0x00fb); // RFC 1995
	/// transfer of an entire zone
	pub const AXFR: Type = Type(0x00fc); // RFC 1035
	/// mailbox-related RRs (MB, MG or MR)
	pub const MAILB: Type = Type(0x00fd); // RFC 1035
	/// mail agent RRs (OBSOLETE - see MX)
	pub const MAILA: Type = Type(0x00fe); // RFC 1035
	/// "*", a request for all records the server/cache has available
	pub const ANY: Type = Type(0x00ff); // RFC 1035

	/// URI
	pub const URI: Type = Type(0x0100); // RFC 7553
	/// Certification Authority Restriction
	pub const CAA: Type = Type(0x0101); // RFC 6844
	/// Application Visibility and Control
	pub const AVC: Type = Type(0x0102); // Wolfgang Riedel
	/// Digital Object Architecture
	pub const DOA: Type = Type(0x0103); // http://www.iana.org/go/draft-durand-doa-over-dns
	/// DNSSEC Trust Authorities
	pub const TA: Type = Type(0x8000); //
	/// DNSSEC Lookaside Validation
	pub const DLV: Type = Type(0x8001); // RFC 4431
}
