# User Scripts

This is the main point of customization. With user scripts you can decide what, where and when to
display. This can be achieved by a combination of built-in data types, using predicates and
subscribing to events.

## Events

Events are at the core of the OmniLED architecture, and it's good to know how they work before
registering keyboard shortcuts and user scripts.

### Event Loop

The main event loop is actually synchronous, and it groups events into batches that are processed
in the interval specified in the [settings](settings.md#update-interval-tick-duration). OmniLED
will wait for the update interval's duration and put keyboard and application update events into a
queue. One the wait is done, it will first process the [keyboard events](#keyboards-events),
triggering the registered shortcuts. Then it will process the
[application update events](#application-update-events), executing the user scripts. If a user
script was executed, the returned layout is then rendered and send to the device.

### Application Update Events

> Note: Data field names are not strictly enforced, but when creating your own application it's
> best to keep the current convention of using PascalCase for data field names.

Application update events are generated when any application sends an update to the server. OmniLED
will then generate an update event for every top level field in the update data. It will also
convert this data to a global variable that is accessible from user scripts.

Additionally, for each update cycle there will be special event called `OMNILED.Update`, so that an
action can be run on each event loop update, rather than relying on receiving application updates
that regularly.

> Example:
>
> Let's suppose the server just got this data from 'MY_APPLICATION' application.
>
> ```lua
> MY_APPLICATION = {
>   Name = "OmniLED",
>   HoursToComplete = "TooMany",
>   SomeExampleData = {
>       a = 0,
>       b = 1,
>   },
> }
> ```
>
> This message will result in three application update events: `MY_APPLICATION.Name`,
> `MY_APPLICATION.HoursToComplete`, `MY_APPLICATION.SomeExampleData`.  
> It will also make `MY_APPLICATION` a global variable that can be accessed by all user scripts.

### Keyboards Events

> _Note: Keyboard events can currently only be used to register for shortcuts, and cannot be used
to trigger user scripts._

When you press a key on the keyboard a new event is generated with a following name 
`"KEY(<key_name>)"`. This event will be generated again under 2 circumstances:

1) You let go of the key, and then press it again
2) You continue to press the key for multiple durations of the update interval (Initial and repeat
delay can be adjusted in [settings](settings.md#keyboard)).

## Drawing on The Screen

Drawing on the screen is as simple as laying out the desired [widgets](#widgets) on the screen and
filling them with the desired data received from the applications.

### Widgets

OmniLED provides a few [widgets](lua_interfaces.md#widgets) that are the building blocks for
everything that can be shown on screen.  
Currently they include:

- [bar](lua_interfaces.md#bar) - Show percentage progress bar
- [image](lua_interfaces.md#image) - Show image
- [text](lua_interfaces.md#text) - Show text
- _More widgets are planned and will be coming soon™️._

Each widget's size and position can be changed independently, and they will scale themselves and
the content to best fit the set size.

### Layouts

[Layouts](lua_interfaces.md#layout) start to tie the functionalities together. Layout provides a
function that returns [layout data](lua_interfaces.md#layoutdata), that consists of a widget list,
display duration and [repeat strategy](lua_interfaces.md#repeat). Layout also registers for events
that will trigger it, as well as a predicate that will be called if a matching event is found, for
a more fine-grained control.

> _Note: Widgets are rendered in the order they are provided in the list. This means that later
> widgets can be drawn over the earlier widgets._

### Layout Groups

Layouts can be grouped to form some cohesive structure. This is useful when there you want to show
a base layout most of the time, and show another event when it becomes available, e.g. a clock most
of the time and a notification when you change speaker volume.

If layouts are not well suited for above scenario, they can be put into separate layout groups.
That way you can decide to switch between layout groups with a keyboard shortcut, when you want to
see different data.

### Layout Priorities

How does OmniLED decide which layout to show on the screen if multiple layouts registered for the same event?

First it checks the priority and remaining display time of the previously rendered layout. Then it
goes from the first layout in the list (highest priority) to the last one (lowest priority) and see
if any of them match the event. There are 2 cases when the matched layout will actually be rendered:

1) Matched layout with greater or equal priority to the previous one
2) Matched any layout and the remaining display time is zero

OmniLED will only consider layouts that are in the active layout group. Layouts in other groups
will be ignored.

### Putting it all together

[Screen builder](lua_interfaces.md#screen_builder) interface allows to easily create layout groups,
add group switch shortcut, and register them for a given device.

> Example:
>
> Add 2 very simple layout groups to `My Device`.
>
> ```lua 
> function layout_1()
>     return {
>         widgets = {
>             Text {
>                 text = AUDIO.Volume,
>                 position = { x = 0, y = 0 },
>                 size = { width = SCREEN.Width, height = SCREEN.Height },
>             },
>         },
>         duration = 3000,
>     }
> end
> 
> function layout_2()
>     return {
>         widgets = {
>             Text {
>                 text = 'Layout 2',
>                 position = { x = 0, y = 0 },
>                 size = { width = SCREEN.Width, height = SCREEN.Height / 2 },
>             },
>             Text {
>                 text = string.format("%02d:%02d:%02d", CLOCK.Hours, CLOCK.Minutes, CLOCK.Seconds),
>                 position = { x = 0, y = SCREEN.Height / 2 },
>                 size = { width = SCREEN.Width, height = SCREEN.Height / 2 },
>             },
>         },
>         duration = 2000,
>     }
> end
> 
> function layout_3()
>     return {
>         widgets = {
>             Text {
>                 text = string.format("%02d:%02d:%02d", CLOCK.Hours, CLOCK.Minutes, CLOCK.Seconds),
>                 position = { x = 0, y = 0 },
>                 size = { width = SCREEN.Width, height = SCREEN.Height / 2 },
>             },
>             Text {
>                 text = 'Layout 3',
>                 position = { x = 0, y = SCREEN.Height / 2 },
>                 size = { width = SCREEN.Width, height = SCREEN.Height / 2 },
>             },
>         },
>         duration = 1000,
>     }
> end
> 
> SCREEN_BUILDER
>     :new('My Device')
>     :with_layout_group({                  -- This is the initial active layout group
>         {
>             layout = layout_1,            -- This layout is the highest priority in layout group 1
>             run_on = { 'AUDIO.Volume' },
>         },
>         {
>             layout = layout_2,            -- This layout is the lowest priority in layout group 1
>             run_on = { 'CLOCK.Seconds' },
>         },
>     })
>     :with_layout_group({                  -- This layout group can be enabled after pressing the shortcut
>         {
>             layout = layout_3,            -- This is the only layout in layout group 2
>             run_on = { 'CLOCK.Seconds' },
>         }
>     })
>     :with_layout_group_toggle({ 'KEY(RAlt)', 'KEY(Slash)' })
>     :register()
> ```

In the above example we have 3 layouts. `layout_1` and `layout_2` in the first group - this will
always be the active layout group since it was registered first. Then there is the other group with
just `layout_3` - it can only be activated after pressing the shortcut which in this example is
`Right Alt` + `/`.

Let's see what happens when `CLOCK.Seconds` event is received. This is the initial update so no
previous render priority, `layout_2` is thus rendered and the remaining display time is set to
`2000` ms. If another `CLOCK.Seconds` is received after `100` ms it, `layout_2` will be rendered
again and remaining display time will set to `2000` ms again, since it has the same priority. After
`AUDIO.Volume` event, `layout_1` will be rendered and display time will be set to `3000` ms. This
is allowed since this layout was first in the list, so it has higher priority. Now for the next
`3000` ms, receiving `CLOCK.Seconds` will not render `layout_2` as it has lower priority.

Now let's suppose `AUDIO.Volume` event was just received so `layout_1` was just rendered and
remaining display time is set to `3000` ms. After pressing the layout group toggle shortcut,
current priority and remaining time is reset and the next layout group is activated. Now receiving
`CLOCK.Seconds` event will render `layout_3`. Pressing the shortcut again will reset the priority
and remaining time again, and will activate next layout group (in this case it already wrapped
around to the first one).
