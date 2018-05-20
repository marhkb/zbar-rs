use super::*;
use symbol::*;

pub struct SymbolSet {
    symbol_set: *const zbar_symbol_set_s,
}
impl SymbolSet  {
    /// Creates a new `SymbolSet` from raw data.
    pub(crate) fn from_raw(symbol_set: *const zbar_symbol_set_s) -> Option<Self> {
        match !symbol_set.is_null() {
            true  => {
                let mut symbol_set = Self { symbol_set };
                symbol_set.set_ref(1);
                Some(symbol_set)
            },
            false => None,
        }
    }

    /// Increases the reference count.
    fn set_ref(&mut self, refs: i32) { unsafe { zbar_symbol_set_ref(**self, refs) } }

    pub fn size(&self) -> i32 { unsafe { zbar_symbol_set_get_size(**self) } }
    /// Returns the first `Symbol` if one is present.
    ///
    /// # Examples
    ///
    /// ```
    /// use zbars::prelude::*;
    ///
    /// let mut image = ZBarImage::from_owned(1, 1, Format::from_label("Y8"), vec![1]).unwrap();
    /// let mut scanner = ImageScanner::builder().build().unwrap();
    /// if let Ok(symbol_set) = scanner.scan_image(&mut image) {
    ///     match symbol_set.first_symbol() {
    ///         Some(symbol) => println!("{}", symbol.data()),
    ///         None         => println!("no symbols in symbol set"),
    ///     }
    /// };
    /// ```
    pub fn first_symbol(&self) -> Option<Symbol> {
        Symbol::from_raw(unsafe { zbar_symbol_set_first_symbol(**self) })
    }

    pub fn iter(&self) -> SymbolIter { self.first_symbol().into() }
}

impl Deref for SymbolSet {
    type Target = *const zbar_symbol_set_s;
    fn deref(&self) -> &Self::Target { &self.symbol_set }
}

impl Clone for SymbolSet {
    fn clone(&self) -> Self {
        let mut symbol_set = Self { symbol_set: self.symbol_set };
        symbol_set.set_ref(1);
        symbol_set
    }
}

impl Drop for SymbolSet {
    fn drop(&mut self) { self.set_ref(-1) }
}

#[cfg(feature = "zbar_fork")]
pub mod zbar_fork {
    use super::*;

    impl SymbolSet  {
        pub fn first_symbol_unfiltered(&self) -> Option<Symbol> {
            Symbol::from_raw(unsafe { zbar_symbol_set_first_unfiltered(**self) })
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test] fn test_first_symbol_unfiltered() {
            assert_eq!(create_symbol_set().first_symbol_unfiltered().unwrap().data(), "Hello World");
        }
    }
}


pub struct SymbolIter {
    symbol: Option<Symbol>,
}
impl From<Option<Symbol>> for SymbolIter {
    fn from(symbol: Option<Symbol>) -> Self { Self { symbol } }
}
impl Iterator for SymbolIter {
    type Item = Symbol;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next = self.symbol.as_ref().and_then(Symbol::next);
        mem::swap(&mut self.symbol, &mut next);
        next
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test] fn test_size() { assert_eq!(create_symbol_set().size(), 2); }
    #[test] fn test_first_symbol() {
        assert_eq!(create_symbol_set().first_symbol().unwrap().data(), "Hello World");
    }
    #[test] fn test_iter() {
        let symbols = create_symbol_set();
        let mut iter = symbols.iter();
        assert_eq!(iter.next().unwrap().data(), "Hello World");
        assert_eq!(iter.next().unwrap().data(), "Hallo Welt");
        assert!(iter.next().is_none());
    }
}

#[cfg(test)]
fn create_symbol_set() -> SymbolSet {
    create_symbol_from("test/greetings.png").symbols().unwrap()
}
#[cfg(test)]
fn create_symbol_from(path: impl AsRef<::std::path::Path>) -> prelude::ZBarImage<'static> {
    use prelude::{
        ZBarImage,
        ImageScanner
    };

    let mut image = ZBarImage::from_path(&path).unwrap();

    let mut scanner = ImageScanner::builder()
        .with_cache(false)
        .with_config(ZBarSymbolType::ZBAR_QRCODE, ZBarConfig::ZBAR_CFG_ENABLE, 1)
        .with_config(ZBarSymbolType::ZBAR_CODE128, ZBarConfig::ZBAR_CFG_ENABLE, 1)
        .build()
        .unwrap();
    scanner.scan_image(&mut image).unwrap();
    image
}
