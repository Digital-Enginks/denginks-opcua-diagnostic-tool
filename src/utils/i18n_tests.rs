#[cfg(test)]
mod tests {
    use crate::utils::i18n::{self, T, Language};

    #[test]
    fn test_english_translations() {
        assert_eq!(i18n::t(T::File, Language::English), "File");
        assert_eq!(i18n::t(T::Exit, Language::English), "Exit");
        assert_eq!(i18n::t(T::About, Language::English), "About");
    }

    #[test]
    fn test_spanish_translations() {
        assert_eq!(i18n::t(T::File, Language::Spanish), "Archivo");
        assert_eq!(i18n::t(T::Exit, Language::Spanish), "Salir");
        assert_eq!(i18n::t(T::About, Language::Spanish), "Acerca de");
    }

    #[test]
    fn test_security_translations() {
        assert_eq!(i18n::t(T::SecurityNone, Language::English), "None (No Security)");
        assert_eq!(i18n::t(T::SecurityNone, Language::Spanish), "Ninguna (Sin seguridad)");
    }
}
