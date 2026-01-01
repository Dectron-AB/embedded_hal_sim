pub mod adc;
#[cfg(feature = "ethernet")]
pub mod ethernet;
#[cfg(feature = "flash")]
pub mod flash;
pub mod gpio;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
