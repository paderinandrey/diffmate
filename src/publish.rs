#![cfg_attr(not(test), expect(dead_code))]

mod load;
mod post;

#[cfg(test)]
mod tests;

#[cfg_attr(not(test), allow(unused_imports))]
pub(crate) use post::execute_publish;
