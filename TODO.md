* [x] Внести ошереди приёма и отправки внутрь `DiscordEventHandler` и сделать его API предсказуемым
    * [x] get_event().await
    * [x] send_message().await
        `DiscordEventHandler` мувается в клиент, нельзя вызывать методы на нем, остаются только каналы
    * [v] В случае голосового общения, как определить в какой канал слать текстовые ответы? - GuildId
* [v] RegexSet
* [x] Новое API - голосовых сообщений вместо прикерепления файла, возможно-ли?
    Не поддерживается библиотекой (пока)
* [v] Разделение звука на фразы по тишине
* [v] rustyline-async для консоли
* [v] [OpenAI whisper](https://github.com/openai/whisper) service
* [v] Игнорировать голосовые фрагменты длиной менее <настройка> и более <настройка> секунд
    * [v] Настройка
    * [v] Interactive
    * [v] Discord bot
* [v] Голосовые запросы из дискорда отделный тип сообщений
* [v] Тест STT - отдельная программа
* [v] Обновить конфиг. Выбор между LLaMa и OpenAI более чёткий (enum)
* [v] Команда /reset для сброса диалога
* [ ] Долгий второй прогон Silero TTS
* [v] Замена библиотеки wav для python на что-то другое (Она встроена в python3.8+, удалил зависимость)
* [v] Если в дискорде запрос текстовой то выводить "Typing..."
* [v] добавить поддержку Alpaca + Lora
* [v] Добавить конфигурацию LLaMa кроме url
* [ ] Сброс не работает если бот общается только голосом
* [v] Замена LLaMa подсистемы на ChatGPT CustomURL
* [ ] Японский TSS Через https://huggingface.co/spaces/Plachta/VITS-Umamusume-voice-synthesizer/blob/main/app.py
    * [ ] Когда появится Rust клиент для [Gradio](https://gradio.app/) переписать на него
    * [v] Костыль: используя самописный скрипт сделать наподобие Silero TTS
    * [v] Неправильная частота дискретизации, дискорд-бот читает слишком бысто. Нужно сделать ресемплинг в 48000 (src\bin\ai-waifu-discord-bot\discord_event_handler.rs:298)
* [v] Двуязычность. Текстовые запросы на русском, TTS на японском.
    * [v] `DisplayRawResp` если эта настройка не `None` то выводить текстовые напрямую от AI.
    * [v] Дискорд бот будет писать тектовые ответы даже если запрос был голосовой
    * [v] Интерактивный режим будет писать текстовые как получены от AI
* [v] Сохранение сонтекста беседы между запускамми программы
* [v] Если выбран режим `DisplayRawResp` выводить Ответ [Raw].
* [v] Сброс контекста беседы в интерактивном режиме. Команда `/reset`
* [v] Интерактивный режим
    * [v] Текст ответа должен появляться раньше чем голосовой ответ
    * [v] Добавить команду `/repeat` для повтора последнего ответа голосом
    * [ ] `tracing` конфликтует с `rustyline_async` вывод лога затирает строку ввода.
    * [ ] Субтитры
        * [v] Добавить в ключи запуска `subtitles-req` и `subtitles-ans` - путь к файлам с субтитрами
        * [v] Полученый запрос сохранять в файл `subtitles-req`
        * [v] Полученый ответ сохранять в файл `subtitles-ans`
        * [v] По окончании воспроизведения ответа очистить эти файлы