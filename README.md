# Micro Block

Утилита для Windows 10/11, которая блокирует микрофон и проверяет, что запись действительно не ведётся.

Одно окно, одна круглая кнопка: красный микрофон — микрофон активен, зелёный перечёркнутый — заблокирован.

## Что делает

При блокировке применяются два независимых механизма:

| Метод | Что делает |
|---|---|
| Реестр | `HKCU\Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\microphone` → `Deny` |
| Core Audio | mute на всех активных устройствах записи |

Перед изменением состояние сохраняется в `%LOCALAPPDATA%\MicroBlock\state.txt`, поэтому разблокировка работает и после перезапуска.

Разблокировка возвращает только то, что изменила программа: исходное значение реестра, mute и громкость каждого устройства по его ID. Без сохранённого состояния mute не трогается.

Журнал: `%LOCALAPPDATA%\MicroBlock\log.txt`.

Ручной режим: `micro-block.exe --unmute-all` снимает mute со всех устройств записи (аварийный сброс).

## Сборка (debug)

Нужны Rust (stable, MSVC toolchain) и Visual Studio Build Tools с C++.

```
cargo build
cargo run
```

Приложение требует прав администратора: манифест с `requireAdministrator` встроен в exe, Windows покажет UAC при запуске.

## Сборка релиза

```
cargo build --release --target x86_64-pc-windows-msvc
```

С флагами из `.cargo/config.toml` и профилем из `Cargo.toml` это даёт один самодостаточный exe со статической CRT:

```
target\x86_64-pc-windows-msvc\release\micro-block.exe
```

## Проверка

Автотестов пока нет. Проверка вручную: блокировка → реестр `Deny`, устройства записи в mute; разблокировка → всё вернулось; перезапуск в заблокированном состоянии определяется по реестру.

## Структура

```
src/main.rs              точка входа, окно, --unmute-all
src/app.rs               состояния и кнопка
src/icon.rs              иконка микрофона и перечёркивание
src/theme.rs             палитра и шрифты
src/footer.rs            ссылка «Powered by DevUnits»
src/worker.rs            фоновый поток: блокировка, откат, снапшот
src/log.rs               журнал в %LOCALAPPDATA%
src/lock/privacy.rs      реестр
src/lock/core_audio.rs   Core Audio: mute и громкость
app.manifest.xml         манифест UAC
build.rs                 встраивание манифеста и иконки
assets/                  иконка приложения
```

## Лицензия

MIT.

Powered by [DevUnits](https://github.com/DevUnits).
