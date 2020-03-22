#![deny(clippy::all)]
#![deny(clippy::cargo)]

pub mod components;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
