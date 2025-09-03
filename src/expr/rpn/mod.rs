pub mod ifrpn;
pub mod irpn;
pub mod ivrpn;
pub mod rpn;

pub use ifrpn::IFRpnToken;
pub use irpn::IRpnToken;
pub use ivrpn::IVRpnToken;
pub use rpn::RpnToken;

use crate::{ConstantResolver, DefaultResolver, EmptyResolver, SmallResolver, expr::Expr};

trait NotIndexedResolver {}
impl<State, T> NotIndexedResolver for DefaultResolver<State, T> {}
impl<T> NotIndexedResolver for ConstantResolver<T> {}
impl<State, T> NotIndexedResolver for SmallResolver<State, T> {}
impl NotIndexedResolver for EmptyResolver {}

impl<T> Expr<T> {}
