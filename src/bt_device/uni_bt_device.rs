#[derive(Debug)]
pub struct UniBtDevice {
	pub address: Address,
	pub parent_address: Address,
	pub ltk: Ltk,
	pub e_rand: Option<ERand>,
	pub e_div: Option<EDiv>,
	pub irk: Option<Irk>,
	pub csrk: Option<Csrk>,
}

#[derive(Debug, Clone)]
pub struct Address(pub [u8; 6]);

#[derive(Debug, Clone)]
pub struct Ltk(pub [u8; 16]);

#[derive(Debug, Clone)]
pub struct ERand(pub [u8; 8]);

#[derive(Debug, Clone)]
pub struct EDiv(pub [u8; 4]);


#[derive(Debug, Clone)]
pub struct Irk(pub [u8; 16]);

#[derive(Debug, Clone)]
pub struct Csrk(pub [u8; 16]);