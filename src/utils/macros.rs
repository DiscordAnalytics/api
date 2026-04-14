#[macro_export]
macro_rules! impl_kv_iter {
  ($struct:ty, [ $( ($key:expr, $field:ident) ),* $(,)? ]) => {
    impl IntoIterator for $struct {
        type Item = (String, i32);
        type IntoIter = std::vec::IntoIter<Self::Item>;

        fn into_iter(self) -> Self::IntoIter {
            vec![
                $(
                    ($key.to_string(), self.$field),
                )*
            ]
            .into_iter()
        }
    }
  };
}
