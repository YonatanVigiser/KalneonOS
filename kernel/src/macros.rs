#[macro_export]
macro_rules! debug_hex {
  ($struct_name:ident, hex: [$($hex_field:ident),* $(,)?], normal: [$($norm_field:ident),* $(,)?]) => {
    impl fmt::Debug for $struct_name {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!($struct_name))
          $(
              .field(stringify!($hex_field), &format_args!("{:#x}", self.$hex_field))
          )*
          $(
              .field(stringify!($norm_field), &self.$norm_field)
          )*
          .finish()
      }
    }
  };
}

