#[macro_export]
macro_rules! debug_hex {
  ($struct_name:ident, hex: [$($hex_field:ident),* $(,)?], normal: [$($norm_field:ident),* $(,)?]) => {
    impl core::fmt::Debug for $struct_name {
      fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut ds = f.debug_struct(stringify!($struct_name));

        $(
          let val = self.$hex_field;
          ds.field(stringify!($hex_field), &format_args!("{:#x}", val));
        )*

        $(
          ds.field(stringify!($norm_field), &self.$norm_field);
        )*

        ds.finish()
      }
    }
  };
}

