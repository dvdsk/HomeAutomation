mod compression;

pub type FieldId = u8;
pub struct Field<T> {
	pub offset: u8, //bits
	pub length: u8, //bits (max 32 bit variables)
	
	pub decode_scale: T,
	pub decode_add: T,
}

//TODO do away with generics in favor for speeeeed
impl<T> Field<T>
where T: num::cast::NumCast+std::fmt::Display+std::ops::Add+std::ops::SubAssign+std::ops::DivAssign+std::ops::MulAssign+std::marker::Copy {
	pub fn decode<D>(&self, line: &[u8]) -> D
	where D: num::cast::NumCast+std::fmt::Display+std::ops::Add+std::ops::SubAssign+std::ops::MulAssign+std::ops::AddAssign{
	//where D: From<T>+From<u32>+From<u16>+std::ops::Add+std::ops::SubAssign+std::ops::DivAssign+std::ops::AddAssign{
		let int_repr: u32 = compression::decode(line, self.offset, self.length);
		//println!("int regr: {}", int_repr);
		let mut decoded: D = num::cast(int_repr).unwrap();
		
		//println!("add: {}", self.decode_add);
		//println!("scale: {}", self.decode_scale);

		decoded *= num::cast(self.decode_scale).unwrap();//FIXME flip decode scale / and *
		decoded += num::cast(self.decode_add).unwrap();
	
		decoded
	}
	pub fn encode<D>(&self, mut numb: T, line: &mut [u8])
	where D: num::cast::NumCast+std::fmt::Display+std::ops::Add+std::ops::SubAssign+std::ops::AddAssign+std::ops::DivAssign{

		//println!("org: {}",numb);
		numb -= num::cast(self.decode_add).unwrap();
		numb /= num::cast(self.decode_scale).unwrap();
		//println!("scale: {}, add: {}, numb: {}", self.decode_scale, self.decode_add, numb);

		let to_encode: u32 = num::cast(numb).unwrap();

		compression::encode(to_encode, line, self.offset, self.length);
	}
}