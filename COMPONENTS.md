# Components

Current public APIs are below. None has been certified as a complete port yet.
Use `cargo run --bin gallery -- --list` for currently supported capture routes.
The full upstream inventory and example coverage live in reference/components.json.

| Component | Public Rust API | Use |
|---|---|---|
| Accordion | `accordion_item` | A vertically stacked set of interactive headings that each reveal a section of content. |
| Alert | `alert` | Displays a callout for user attention. |
| Alert Dialog | `alert_dialog`, `OverlayState` | A modal dialog that interrupts the user with important content and expects a response. |
| Aspect Ratio | Pending | Displays content within a desired ratio. |
| Attachment | Pending | Displays a file or image attachment with media, metadata, upload state, and actions. |
| Avatar | `avatar` | An image element with a fallback for representing the user. |
| Badge | `badge` | Displays a badge or a component that looks like a badge. |
| Breadcrumb | `breadcrumb` | Displays the path to the current resource using a hierarchy of links. |
| Bubble | Pending | Displays conversational content in a message bubble. Supports variants, alignment, grouping, reactions, and collapsible content. |
| Button | `Button`, `button`, `Variant`, `ButtonSize` | Displays a button or a component that looks like a button. |
| Button Group | `button_group` | A container that groups related buttons together with consistent styling. |
| Calendar | Pending | A calendar component that allows users to select a date or a range of dates. |
| Card | `card`, `card_sized`, `card_header`, `card_title`, `card_description`, `card_content`, `card_footer` | Displays a card with header, content, and footer. |
| Carousel | Pending | A carousel with motion and swipe built using Embla. |
| Chart | Pending | Beautiful charts. Built using Recharts. Copy and paste into your apps. |
| Checkbox | `checkbox`, `checkbox_disabled` | A control that allows the user to toggle between checked and not checked. |
| Collapsible | `collapsible` | An interactive component which expands/collapses a panel. |
| Combobox | Pending | Autocomplete input with a list of suggestions. |
| Command | Pending | Command menu for search and quick actions. |
| Context Menu | Pending | Displays a menu of actions triggered by a right click. |
| Data Table | Pending | Powerful table and datagrids built using TanStack Table. |
| Date Picker | Pending | A date picker component with range and presets. |
| Dialog | `dialog`, `OverlayState` | A window overlaid on either the primary window or another dialog window, rendering the content underneath inert. |
| Direction | Pending | A provider component that sets the text direction for your application. |
| Drawer | `drawer` | A drawer component for React. |
| Dropdown Menu | Pending | Displays a menu to the user — such as a set of actions or functions — triggered by a button. |
| Empty | `empty` | Use the Empty component to display an empty state. |
| Field | `field` | Combine labels, controls, and help text to compose accessible form fields and grouped inputs. |
| Form | Pending |  |
| Hover Card | `hover_card` | For sighted users to preview content available behind a link. |
| Input | `input` | A text input component for forms and user data entry with built-in styling and accessibility features. |
| Input Group | Pending | Add addons, buttons, and helper content to inputs. |
| Input OTP | Pending | Accessible one-time password component with copy-paste functionality. |
| Item | Pending | A versatile component for displaying content with media, title, description, and actions. |
| Kbd | `kbd` | Used to display textual user input from keyboard. |
| Label | `label` | Renders an accessible label associated with controls. |
| Marker | Pending | Displays an inline status, system note, bordered row, or labeled separator in a conversation. |
| Menubar | Pending | A visually persistent menu common in desktop applications that provides quick access to a consistent set of commands. |
| Message | Pending | Displays a message in a conversation, with optional avatar, header, footer, and alignment. |
| Message Scroller | Pending | A chat scroll container that anchors turns, opens saved transcripts, follows streamed responses, loads history without jumping, and jumps to any message. |
| Native Select | Pending | A styled native HTML select element with consistent design system integration. |
| Navigation Menu | Pending | A collection of links for navigating websites. |
| Pagination | `pagination` | Pagination with page navigation, next and previous links. |
| Popover | `popover` | Displays rich content in a portal, triggered by a button. |
| Progress | `progress` | Displays an indicator showing the completion progress of a task, typically displayed as a progress bar. |
| Questionnaire | Pending | A multi-step questionnaire with single-choice, multiple-choice, freeform, and skippable questions. |
| Radio Group | `radio_group` | A set of checkable buttons—known as radio buttons—where no more than one of the buttons can be checked at a time. |
| Resizable | Pending | Accessible resizable panel groups and layouts with keyboard support. |
| Scroll Area | `scroll_area` | Augments native scroll functionality for custom, cross-browser styling. |
| Select | Pending | Displays a list of options for the user to pick from—triggered by a button. |
| Separator | `separator` | Visually or semantically separates content. |
| Sheet | `sheet`, `Side` | Extends the Dialog component to display content that complements the main content of the screen. |
| Sidebar | Pending | A composable, themeable and customizable sidebar component. |
| Skeleton | `skeleton` | Use to show a placeholder while content is loading. |
| Slider | Pending | An input where the user selects a value from within a given range. |
| Spinner | `spinner` | An indicator that can be used to show a loading state. |
| Switch | `switch` | A control that allows the user to toggle between checked and not checked. |
| Table | Pending | A responsive table component. |
| Tabs | `tabs`, `TabsVariant` | A set of layered sections of content—known as tab panels—that are displayed one at a time. |
| Textarea | `textarea` | Displays a form textarea or a component that looks like a textarea. |
| Toast | Pending | A succinct message that is displayed temporarily. |
| Toggle | `toggle` | A two-state button that can be either on or off. |
| Toggle Group | Pending | A set of two-state buttons that can be toggled on or off. |
| Tooltip | `tooltip` | A popup that displays information related to an element when the element receives keyboard focus or the mouse hovers over it. |
| Typography | `text`, `muted`, `heading` | Styles for headings, paragraphs, lists, etc. |
