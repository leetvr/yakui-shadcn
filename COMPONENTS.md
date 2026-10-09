# Components

Every catalogue entry has an initial native API and gallery route. None is certified
as a complete port. See `reference/components.json` for per-preview coverage, actual
behaviour tests, comparisons and known deviations. `gallery --list` reports supported
native routes; unported upstream preview names return an error.

Use `ui::column` or `ui::row` for multiple sequential children. Generic Yakui containers
overlay children. Keep overlays at the root and retain their state in your application.

| Component | Public API | Use |
|---|---|---|
| Accordion | `accordion_item` | A vertically stacked set of interactive headings that each reveal a section of content. |
| Alert | `alert` | Displays a callout for user attention. |
| Alert Dialog | `alert_dialog`, `OverlayState` | A modal dialog that interrupts the user with important content and expects a response. |
| Aspect Ratio | `aspect_ratio` | Displays content within a desired ratio. |
| Attachment | `Attachment`, `AttachmentState`, `AttachmentSize`, `attachment` | Show file metadata, upload state and actions. |
| Avatar | `Avatar`, `AvatarImage`, `AvatarSize`, `AvatarBadge`, `avatar_group`, `AvatarGroupCount`, `avatar` | Display a circular application texture or initials, with status badges and overlapping groups. |
| Badge | `badge` | Displays a badge or a component that looks like a badge. |
| Breadcrumb | `breadcrumb` | Displays the path to the current resource using a hierarchy of links. |
| Bubble | `bubble`, `BubbleVariant` | Group conversational content on a themed surface. |
| Button | `Button`, `button`, `Variant`, `ButtonSize`, `link` | Displays a button or a component that looks like a button. |
| Button Group | `ButtonGroup`, `button_group`, `button_group_text`, `button_group_separator` | A container that groups related buttons together with consistent styling. |
| Calendar | `Date`, `CalendarState`, `CalendarMode`, `calendar` | A calendar component that allows users to select a date or a range of dates. |
| Card | `card`, `card_sized`, `card_header`, `card_header_with_action`, `card_title`, `card_description`, `card_content`, `card_footer` | Displays a card with header, content, and footer. |
| Carousel | `CarouselState`, `CarouselOptions`, `carousel` | Navigate or swipe through composable slides. |
| Chart | `Chart`, `ChartKind`, `ChartPoint`, `ChartSeries`, `ChartResponse` | Plot series with native meshes and inspect points. |
| Checkbox | `Checkbox`, `checkbox`, `checkbox_disabled` | A control that allows the user to toggle between checked and not checked. |
| Collapsible | `collapsible` | An interactive component which expands/collapses a panel. |
| Combobox | `SelectState`, `combobox`, `combobox_content` | Autocomplete input with a list of suggestions. |
| Command | `command`, `MenuItem` | Command menu for search and quick actions. |
| Context Menu | `context_menu_target`, `context_menu`, `MenuState` | Displays a menu of actions triggered by a right click. |
| Data Table | `DataTableState`, `TableColumn`, `TableRow`, `data_table` | Filter, sort, select and page application rows. |
| Date Picker | `date_picker`, `date_picker_content`, `CalendarState`, `OverlayState` | A date picker component with range and presets. |
| Dialog | `dialog`, `OverlayState` | A window overlaid on either the primary window or another dialog window, rendering the content underneath inert. |
| Direction | `direction`, `ReadingDirection` | Mirror inline row layout within a subtree. |
| Drawer | `drawer` | Show a bottom modal; dragging remains pending. |
| Dropdown Menu | `dropdown_menu`, `MenuItem`, `MenuKind`, `MenuState` | Displays a menu to the user — such as a set of actions or functions — triggered by a button. |
| Empty | `empty` | Use the Empty component to display an empty state. |
| Field | `field`, `field_label`, `field_description`, `field_error` | Compose labels, controls, help text and validation errors. |
| Form | `form`, `field`, `Input` | Compose controls into an application-owned form. |
| Hover Card | `hover_card` | For sighted users to preview content available behind a link. |
| Input | `Input`, `InputResponse`, `input` | A text input component for forms and user data entry with built-in styling and accessibility features. |
| Input Group | `InputGroup`, `InputGroupAlign`, `input_group`, `input_group_addon`, `input_group_input`, `input_group_textarea`, `input_group_button`, `input_group_text` | Add addons, buttons, and helper content to inputs. |
| Input OTP | `input_otp`, `OtpOptions`, `OtpPattern`, `OtpResponse` | Accessible one-time password component with copy-paste functionality. |
| Item | `item` | A versatile component for displaying content with media, title, description, and actions. |
| Kbd | `kbd` | Used to display textual user input from keyboard. |
| Label | `label` | Draw a medium-weight label; association APIs remain pending. |
| Marker | `marker`, `MarkerVariant` | Displays an inline status, system note, bordered row, or labeled separator in a conversation. |
| Menubar | `menubar`, `MenubarState` | A visually persistent menu common in desktop applications that provides quick access to a consistent set of commands. |
| Message | `message`, `MessageAlign` | Displays a message in a conversation, with optional avatar, header, footer, and alignment. |
| Message Scroller | `MessageScrollerState`, `message_scroller`, `message_scroller_item` | A chat scroll container that anchors turns, opens saved transcripts, follows streamed responses, loads history without jumping, and jumps to any message. |
| Native Select | `native_select`, `select_content`, `SelectState` | Select a value through a native Yakui popup. |
| Navigation Menu | `NavigationMenuEntry`, `NavigationMenuState`, `NavigationMenuLink`, `navigation_menu_bar`, `navigation_menu_content`, `navigation_menu` | Open richer navigation flyouts or return route IDs to your app. |
| Pagination | `pagination` | Pagination with page navigation, next and previous links. |
| Popover | `popover` | Displays rich content in a portal, triggered by a button. |
| Progress | `progress` | Displays an indicator showing the completion progress of a task, typically displayed as a progress bar. |
| Questionnaire | `Question`, `QuestionChoice`, `Answer`, `QuestionnaireState`, `QuestionnaireResponse`, `questionnaire` | A multi-step questionnaire with single-choice, multiple-choice, freeform, and skippable questions. |
| Radio Group | `radio_group` | A set of checkable buttons—known as radio buttons—where no more than one of the buttons can be checked at a time. |
| Resizable | `resizable`, `Orientation` | Accessible resizable panel groups and layouts with keyboard support. |
| Scroll Area | `scroll_area` | Augments native scroll functionality for custom, cross-browser styling. |
| Select | `Select`, `SelectItem`, `SelectSize`, `SelectContent`, `SelectState` | Pick an application value from grouped, scrollable choices. |
| Separator | `separator` | Visually or semantically separates content. |
| Sheet | `sheet`, `Side` | Extends the Dialog component to display content that complements the main content of the screen. |
| Sidebar | `SidebarLink`, `SidebarState`, `sidebar` | A composable, themeable and customizable sidebar component. |
| Skeleton | `skeleton`, `skeleton_at` | Use to show a placeholder while content is loading. |
| Slider | `slider`, `range_slider`, `SliderOptions`, `SliderResponse`, `Orientation` | An input where the user selects a value from within a given range. |
| Spinner | `spinner` | An indicator that can be used to show a loading state. |
| Switch | `Switch`, `SwitchSize`, `switch` | A control that allows the user to toggle between checked and not checked. |
| Table | `Table`, `TableCell`, `table`, `TableColumn`, `TableRow` | A responsive table component. |
| Tabs | `tabs`, `TabsVariant` | A set of layered sections of content—known as tab panels—that are displayed one at a time. |
| Textarea | `textarea` | Displays a form textarea or a component that looks like a textarea. |
| Toast | `Toast`, `ToastKind`, `ToastManager`, `ToastResponse`, `toast_viewport` | A succinct message that is displayed temporarily. |
| Toggle | `toggle` | A two-state button that can be either on or off. |
| Toggle Group | `ToggleGroupState`, `toggle_group` | A set of two-state buttons that can be toggled on or off. |
| Tooltip | `tooltip` | A popup that displays information related to an element when the element receives keyboard focus or the mouse hovers over it. |
| Typography | `text`, `muted`, `heading` | Styles for headings, paragraphs, lists, etc. |

## Common patterns

```rust
use yakui_adequate_ui as ui;
ui::width(420., || ui::card(|| {
    ui::card_header(|| {
        ui::card_title("Profile");
        ui::card_description("Your public identity");
    });
    ui::card_content(|| ui::form(|| {
        ui::field("Name", "Shown to other users.", None, || {
            ui::input(&mut profile.name, "Name");
        });
        ui::checkbox("Email updates", &mut profile.updates);
    }));
    ui::card_footer(|| {
        if ui::button("Save").clicked { save(&profile); }
    });
}));
```

Declare triggers in ordinary content and popup content afterwards at the root:

```rust
let trigger = ui::Button::new("Edit profile").variant(ui::Variant::Outline).show();
if trigger.clicked { popup.open_from(trigger); }
ui::dialog(&mut popup, |popup| {
    ui::card_title("Edit profile");
    ui::input(&mut profile.name, "Name");
    if ui::button("Done").clicked { popup.close(); }
});
```

```rust
let choices = [
    ui::SelectItem::label("Fruit"),
    ui::SelectItem::empty("Select a fruit"),
    ui::SelectItem::new(10, "Apple"),
    ui::SelectItem::new(20, "Banana"),
    ui::SelectItem::new(30, "Grapes").disabled(true),
];
ui::Select::new(&choices).placeholder("Select a fruit")
    .width(192.).show(&mut selection);
// At the root, after ordinary content:
if ui::SelectContent::new(&choices).show(&mut selection) {
    // Values are application IDs, not positions in the list.
    save_fruit(selection.selected);
}
```

`SelectState` owns the selected value and popup visibility. `SelectItem::empty`
clears the value; labels and separators cannot be selected. Arrow keys, Home/End
and typeahead move focus, Enter commits, and Escape dismisses. Tab closes and
moves to the next control. The popup aligns the current choice with its trigger;
use `.align_item_with_trigger(false)` to place it below. `.size(SelectSize::Sm)`,
`.disabled(true)` and `.invalid(true)` configure the trigger. For a currency
symbol, `.value_text("$")` overrides its display while retaining the full choices.

The standalone `settings`, `dashboard` and `checkout` examples under
`crates/gallery/examples` compile as separate applications consuming this public API.
Run `cargo run -p adequate-gallery --example settings`.
The gallery includes larger `--showcase dashboard|settings|checkout` compositions.

Checkbox fields and selectable cards share one hit target and focus stop. Boolean
state belongs to the caller. Helper text wraps within the enclosing width.

```rust
ui::width(288., || {
    ui::Checkbox::new()
        .label("Enable notifications")
        .description("You can change this at any time.")
        .card(true)
        .disabled(!settings.editable)
        .invalid(settings.needs_consent)
        .show(&mut settings.notifications);
});
```

Use `Checkbox::new().show(&mut selected)` for a bare table selection control.
`field_legend(title, FieldLegendVariant::Label)` and `field_description(text)`
provide the field-set typography used by checkbox groups.

Compose tables with public slots when cells contain controls. Column widths include
8px padding on each side; the table scrolls horizontally when its parent is narrower.
Captions render below the table. `table(columns, rows, caption)` and `data_table` use
these same slots internally.

```rust
ui::Table::new(&[32., 240.]).show(|| {
    ui::table_header(|| ui::table_row(false, || {
        ui::table_head("");
        ui::table_head("Name");
    }));
    ui::table_body(|| ui::table_row(selected, || {
        ui::table_cell(|| { ui::Checkbox::new().show(&mut selected); });
        ui::table_cell(|| ui::table_cell_text("Sarah Chen", true));
    }));
});
```

Use `TableCell::new().span(3)` for footer totals and
`.align(TableCellAlign::End)` for right-aligned values or action buttons.

Use a switch for an immediately applied setting. It accepts the same caller-owned
boolean as `switch(label, value, disabled)`, with explicit size, validation and
field composition. A trailing switch is placed at the end of the available width.

```rust
ui::width(384., || {
    ui::Switch::new()
        .label("Share across devices")
        .description("Focus is shared across devices.")
        .trailing(true)
        .card(true)
        .show(&mut settings.shared);
});
ui::Switch::new().size(ui::SwitchSize::Sm).show(&mut compact_setting);
```

Compose dropdown triggers around ordinary buttons. Render their menus later at the
application root, outside table/scroll clipping. Pointer opening focuses the popup;
ArrowDown/Up enters its first/last enabled row. Checkbox and radio items stay open
by default, while ordinary action items close and return focus to their trigger.

```rust
ui::dropdown_menu_trigger(&mut menu, || {
    ui::Button::new("").variant(ui::Variant::Ghost)
        .size(ui::ButtonSize::Icon).icon(ui::Icon::MoreHorizontal).show()
});
// At the root, after content:
let action = ui::dropdown_menu_with_options(&mut menu, &mut items, ui::MenuOptions {
    width: 128., align: ui::PopoverAlign::End, ..Default::default()
});
```

`PopoverOptions` and `popover_with_options` expose the same side, alignment and
offset vocabulary for ordinary popovers.

Use `ui::icon(ui::Icon::Settings, 16.)` for the pinned Lucide vocabulary.
`Icon::ALL` lists the 137 assets used by upstream examples; `Icon::name()` returns
the source name. `icon_colored` takes an application colour. The library paints
vectors directly; it has no runtime SVG or browser dependency.

Dropdowns use their trigger width with a 128px minimum by default. Set
`MenuOptions.width` for a fixed popup width; child menus size themselves from
their actual text, icons and shortcuts. `MenuItem::submenu` owns its child items,
while checkbox/radio values live in the caller's item tree. Actions return a
stable ID; use that ID to mutate application state. Root and child menus scroll
and reveal the keyboard selection. English shortcut symbols have a bundled font
fallback.

Applications own image loading and Yakui textures. Keep the texture alive while
an avatar refers to it; pass `None` during loading or on failure.

```rust
ui::Avatar::new("CN")
    .image(profile_image) // Option<ui::AvatarImage>
    .size(ui::AvatarSize::Lg)
    .badge(ui::AvatarBadge::new())
    .show();
ui::avatar_group(|| {
    ui::Avatar::new("CN").image(profile_image).show();
    ui::Avatar::new("ER").show();
    ui::AvatarGroupCount::new("+3").show();
});
ui::Button::new("").variant(ui::Variant::Ghost)
    .size(ui::ButtonSize::Icon).rounded(true)
    .show_children(|| ui::Avatar::new("CN").image(profile_image).show());
```

`AvatarImage::new(texture_id, pixel_dimensions)` crops the image to cover the
circle. The library has no file/image decoder dependency. Run the independent
`avatars` example for app-owned decoding, texture registration and fallback changes.

Use button groups for adjoining actions or an input with a suffix button. Each
control keeps its own focus and click response; groups only join surfaces and
lay out their children. Nest groups for an 8px gap between sets of actions.

```rust
ui::button_group(|| {
    ui::width(240., || { ui::input(&mut query, "Search..."); });
    if ui::Button::new("").variant(ui::Variant::Outline)
        .icon(ui::Icon::Search).show().clicked { search(&query); }
});
ui::ButtonGroup::new().orientation(ui::Orientation::Vertical).show(|| {
    ui::Button::new("").size(ui::ButtonSize::Icon).icon(ui::Icon::Plus).show();
    ui::Button::new("").size(ui::ButtonSize::Icon).icon(ui::Icon::Minus).show();
});
```

`button_group_separator` follows the group's axis. `button_group_text` adds a
bordered prefix or suffix. Use `popover_header`, `popover_title` and
`popover_description` for compact popup text. Inside columns, use `fill_width`
around editors when they should occupy the popup's available width.

Input groups share one surface and focus ring. Declare one editor and addons
directly inside the group. Addons are placed by alignment, independently of their
declaration order; clicking a non-action addon focuses the editor. Embedded buttons
keep their own click and keyboard responses.

```rust
ui::width(320., || ui::input_group(|| {
    ui::input_group_input(&mut query, "Search...");
    ui::input_group_addon(ui::InputGroupAlign::InlineStart, || {
        ui::icon(ui::Icon::Search, 16.);
    });
    ui::input_group_addon(ui::InputGroupAlign::InlineEnd, || {
        if ui::input_group_button("Search").show().clicked { search(&query); }
    });
}));
```

Use `BlockStart`/`BlockEnd` for headers/footers around multiline editors.
`ui::spacer()` before a footer button consumes the row's remaining width.
`InputGroup::new().disabled(true)` disables the editor; action buttons can be
independently disabled. `Input::new(...).password(true)` also shares the group
surface. `input_group_button` returns an ordinary configurable Button: default
Xs is 24px high, Default is 32px, and IconXs supplies a 24px icon action.

Navigation menus keep visibility in application state and return stable route IDs.
Render the bar in content, and its flyout later at the root. Triggers open on hover,
click, Enter or ArrowDown; Escape and outside clicks close the flyout. Sibling
triggers remain reachable while it is open.

```rust
let route = ui::navigation_menu_bar(&[
    ui::NavigationMenuEntry::trigger(1, "Projects"),
    ui::NavigationMenuEntry::link(2, "Settings"),
], &mut navigation);
// Later at the root:
let project = ui::navigation_menu_content(&mut navigation, 320., |_| {
    ui::NavigationMenuLink::new("Current project")
        .description("Open the project's dashboard.")
        .show().clicked.then_some(10)
});
```

Handle the returned IDs in your router. `NavigationMenuState::active()` returns
an open entry's ID; use it to choose content width. `description_lines(2)` clips
long descriptions. `navigation_menu` remains a simple row of direct route actions.
Hover delays, departure closing and animation are deferred for this prototype API.
