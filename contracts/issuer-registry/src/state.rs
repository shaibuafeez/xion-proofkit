use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use proofkit_types::issuer_registry::IssuerRecord;

/// Contract admin address.
pub const ADMIN: Item<Addr> = Item::new("admin");

/// Issuer records indexed by issuer address.
pub const ISSUERS: Map<&Addr, IssuerRecord> = Map::new("issuers");

/// Secondary index: (credential_type, issuer_addr) → () for querying issuers by type.
pub const ISSUERS_BY_TYPE: Map<(&str, &Addr), ()> = Map::new("issuers_by_type");
