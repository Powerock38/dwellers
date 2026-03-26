use bevy::{platform::collections::HashMap, prelude::*};

/// Активный язык. Меняется через `App::insert_resource(ActiveLang("en"))`.
#[derive(Resource)]
pub struct ActiveLang(pub &'static str);

impl Default for ActiveLang {
    fn default() -> Self {
        Self("ru")
    }
}

/// Ресурс локализации. Загружается синхронно при старте из assets/locale/<lang>.ron.
///
/// Формат файла: строки вида `ключ = значение`, комментарии начинаются с `#`.
#[derive(Resource, Default)]
pub struct Locale {
    strings: HashMap<String, String>,
}

impl Locale {
    /// Возвращает перевод по ключу. Если ключ не найден — возвращает сам ключ.
    pub fn t(&self, key: &str) -> String {
        self.strings
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }
}

/// Парсит формат `ключ = значение` (комментарии — строки с `#`).
fn parse_locale(content: &str) -> HashMap<String, String> {
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (k, v) = line.split_once(" = ")?;
            Some((k.trim().to_string(), v.trim().to_string()))
        })
        .collect()
}

/// Startup-система: читает файл локали и заполняет ресурс `Locale`.
///
/// Порядок: должна запускаться **до** `spawn_ui`.
pub fn load_locale(lang: Res<ActiveLang>, mut locale: ResMut<Locale>) {
    let path = format!("assets/locale/{}.ron", lang.0);

    match std::fs::read_to_string(&path) {
        Ok(content) => {
            locale.strings = parse_locale(&content);
            info!("Локаль загружена: {} ({} строк)", path, locale.strings.len());
        }
        Err(e) => {
            warn!("Не удалось загрузить локаль '{}': {}. Используется fallback en.", path, e);

            // Пробуем английский как запасной вариант
            let fallback = format!("assets/locale/{}.ron", "en");
            if let Ok(content) = std::fs::read_to_string(&fallback) {
                locale.strings = parse_locale(&content);
                info!("Загружен fallback: {}", fallback);
            }
        }
    }
}

/// Макрос для удобного получения перевода.
///
/// # Пример
/// ```rust
/// let label = t!(locale, "task.dig");        // String
/// let label = t!(locale, "object.{}", name); // с форматированием ключа
/// ```
#[macro_export]
macro_rules! t {
    ($locale:expr, $key:expr) => {
        $locale.t($key)
    };
    ($locale:expr, $fmt:literal, $($arg:expr),+) => {
        $locale.t(&format!($fmt, $($arg),+))
    };
}
