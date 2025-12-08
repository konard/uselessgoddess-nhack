# NeuroHack Русская локализация
# Этот файл содержит все строки для русского языка.

## Заголовки виджетов
ui-dungeon-title = Подземелье
ui-messages-title = Сообщения
ui-status-title = Статус
ui-controls-title = Управление
ui-look-title = Осмотр
ui-inventory-title = Инвентарь

## Отображение статуса
ui-hp-label = ОЗ
ui-floor-label = Этаж
ui-gold-label = Золото
ui-hp-display = ОЗ: { $current }/{ $max }
ui-floor-display = Этаж: { $floor }
ui-gold-display = Золото: { $gold }

## Режим осмотра
ui-look-mode-header = -- РЕЖИМ ОСМОТРА --
ui-position-label = Позиция
ui-position-display = Позиция: ({ $x }, { $y })
ui-tile-label = Клетка
ui-tile-display = Клетка: { $tile }
ui-not-in-view = Не видно
ui-unexplored = Неизведано

## Названия клеток
tile-wall = Стена
tile-floor = Пол
tile-door = Дверь
tile-stairs-down = Лестница вниз
tile-stairs-up = Лестница вверх

## Справка по управлению
ctrl-move = hjkl/стрелки: Движение
ctrl-move-cursor = hjkl/стрелки: Курсор
ctrl-wait = .: Ждать
ctrl-look = x: Осмотр
ctrl-select = Enter/Пробел: Выбрать
ctrl-exit-look = x/Esc: Выйти из осмотра
ctrl-inventory = i: Инвентарь
ctrl-quit = q: Выход

## Инвентарь
inv-empty = Ваш инвентарь пуст.
inv-close-hint = Нажмите 'i' или Esc чтобы закрыть.

## Боевые сообщения
combat-bump-wall = Вы врезались в стену.
combat-fumble = { $attacker } промахивается по { $defender }!
combat-critical = { $attacker } наносит КРИТИЧЕСКИЙ УДАР по { $defender }! { $damage } ед. урона ({ $type })!
combat-hit = { $attacker } попадает по { $defender }: { $damage } ед. урона ({ $type }).
combat-miss = { $attacker } атакует { $defender }, но промахивается. ({ $roll } против КЗ { $ac })
combat-slain = { $target } повержен!

## Имена существ
entity-you = Вы
entity-you-lowercase = вас
entity-something = Нечто
entity-something-lowercase = нечто

## Типы урона
damage-bludgeoning = дробящий
damage-piercing = колющий
damage-slashing = рубящий
damage-fire = огненный
damage-cold = холод
damage-lightning = молния
damage-acid = кислота
damage-poison = яд
damage-necrotic = некротический
damage-radiant = сияющий
damage-force = силовой
damage-psychic = психический
damage-thunder = грома

## Конец игры
game-over-title = Конец игры
game-over-message = Вы погибли в подземелье.
game-over-restart = Нажмите любую клавишу для перезапуска...
