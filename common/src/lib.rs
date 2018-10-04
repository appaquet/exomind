
extern crate multiaddr;

pub mod node;
pub mod security;
pub mod simplestore;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
