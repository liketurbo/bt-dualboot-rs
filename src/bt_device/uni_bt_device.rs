pub(crate) struct UniBtDevice {
	ltk: Ltk,
	e_rand: ERand,
	e_div: EDiv,
	address: Address,
	parent_address: Address,
	irk: Irk,
	csrk: Csrk,
}

pub(crate) struct Ltk(pub [u8; 16]);

pub(crate) struct ERand(pub [u8; 8]);

pub(crate) struct EDiv(pub [u8; 4]);

pub(crate) struct Address(pub [u8; 6]);

pub(crate) struct Irk(pub [u8; 16]);

pub(crate) struct Csrk(pub [u8; 16]);